using System.Text;
using System.Xml.Linq;
using Clippit;
using Clippit.Word;
using DocumentFormat.OpenXml.Packaging;

namespace libkampfrichtereinsatzplaene_docx;

public class PDFWriter
{

    private string m_savePath;
    private string m_generatedDocx;
    
    public PDFWriter(string savePath, string generatedDocx)
    {
        this.m_savePath = savePath;
        this.m_generatedDocx = generatedDocx;
    }

    public ApplicationError WriteToPDF()
    {
        
        // Get file info
        FileInfo generatedDocxInfo = new FileInfo(this.m_generatedDocx);
        FileInfo pdfToCreate = new FileInfo(this.m_savePath);
        FileInfo htmlToCreate = new FileInfo(pdfToCreate.FullName.Replace(".pdf", "_temp.html"));
        
        // Temp Variables
        string imageDirectory;
        int imageCounter = 0;
        string pageTitle = htmlToCreate.FullName;

        using (WordprocessingDocument wDocument = WordprocessingDocument.Open(generatedDocxInfo.FullName, true))
        {
            if (string.IsNullOrEmpty(this.m_savePath))
            {
                return ApplicationError.CSharpPDFSavePathIsEmpty;
            }

            imageDirectory = htmlToCreate.FullName.Substring(0, htmlToCreate.FullName.Length - 5) + "_files";
            CoreFilePropertiesPart? part = wDocument.CoreFilePropertiesPart;
            if (part is not null)
            {
                pageTitle = (string)part.GetXDocument().Descendants(DC.title).FirstOrDefault() ?? htmlToCreate.FullName;
            }

            WmlToHtmlConverterSettings settings = new WmlToHtmlConverterSettings
            {
                AdditionalCss =
                    "body {display: flex;flex-direction: column; align-items: center;max-width: calc(21cm - 1.27cm - 1.27cm); margin: 0.5cm 1.27cm 0.5cm 1.27cm; }header {display: flex;flex-direction: row-reverse; align-items: start;justify-content: space-between; height: 3cm;width: 100%;max-height: 3cm;padding-top: 1cm;padding-bottom: 0.5cm; }span {white-space: normal !important; }#img0 {height: 3.5cm;}#img1 {height: 1.5cm} p {margin:0;}",
                PageTitle = pageTitle,
                FabricateCssClasses = true,
                CssClassPrefix = "pt-",
                RestrictToSupportedLanguages = false,
                RestrictToSupportedNumberingFormats = false,
                ImageHandler = imageInfo =>
                {
                    ++imageCounter;
                    return ImageHelper.DefaultImageHandler(imageInfo, imageDirectory, imageCounter);
                }
            };
            XElement? htmlElement = WmlToHtmlConverter.ConvertToHtml(wDocument, settings);
            XDocument? html = new XDocument(new XDocumentType("html", null, null, null), htmlElement);

            // OpenXML PowerTools does not yet support Headers and Footers. We have to create them manually.
            XElement body = html.Descendants(Xhtml.body).First();

            // First element in the body should always be the header
            // So add it
            // Create new Header instance
            XElement headerFirst = new XElement("header");

            // Track no of images
            int iFirst = 0;

            // Get images
            // Get the media we need
            if (wDocument.MainDocumentPart is null) return ApplicationError.CSharpWriteError;
            List<ImagePart> images = wDocument.MainDocumentPart.HeaderParts.First().ImageParts.ToList();

            foreach (ImagePart imagePart in images)
            {
                if (imagePart.ContentType == "image/svg+xml")
                {
                    // Get a Base64 representation of the stream!
                    Stream imageStream = imagePart.GetStream();
                    string base64Image = Convert.ToBase64String(imageStream.ReadToArray());
                    imageStream.Close();

                    // Create an img element for this
                    XElement imgElement = new XElement(
                        Xhtml.img,
                        new XAttribute(NoNamespace.src, "data:image/svg+xml;base64, " + base64Image),
                        new XAttribute(NoNamespace.id, "img" + iFirst)
                    );

                    // Add to the header XElement
                    headerFirst.Add(imgElement);

                    // Increment image counter
                    iFirst++;
                }
            }

            // Insert header as first body element
            body.AddFirst(headerFirst);

            var pageBreaks = body.Descendants(Xhtml.div).Where(element =>
                element.Attribute("style") is not null
                    ? element.Attribute("style").Value == "page-break-before: always;"
                    : false);

            foreach (XElement pageBreak in pageBreaks)
            {
                // Create new Header instance
                XElement header = new XElement("header");

                // Track no of images
                int i = 0;

                foreach (ImagePart imagePart in images)
                {
                    if (imagePart.ContentType == "image/svg+xml")
                    {
                        // Get a Base64 representation of the stream!
                        Stream imageStream = imagePart.GetStream();
                        string base64Image = Convert.ToBase64String(imageStream.ReadToArray());
                        imageStream.Close();

                        // Create an img element for this
                        XElement imgElement = new XElement(
                            Xhtml.img,
                            new XAttribute(NoNamespace.src, "data:image/svg+xml;base64, " + base64Image),
                            new XAttribute(NoNamespace.id, "img" + i)
                        );

                        // Add to the header XElement
                        header.Add(imgElement);

                        // Increment image counter
                        i++;
                    }
                }

                // Insert header after page break!
                pageBreak.AddAfterSelf(header);
            }

            string htmlString = html.ToString(SaveOptions.DisableFormatting);
            File.WriteAllText(htmlToCreate.FullName, htmlString, Encoding.UTF8);
        }

        return ApplicationError.NoError;
    }
    
}