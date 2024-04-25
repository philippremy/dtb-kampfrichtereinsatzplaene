using System.Text.RegularExpressions;
using DocumentFormat.OpenXml.Packaging;

namespace libkampfrichtereinsatzplaene_docx;

public partial class DocumentWriter
{

    private string? wkName;
    private string? wkDate;
    private string? wkPlace;
    private string? wkResponsiblePerson;
    private string? wkJudgesMeetingTime;
    private string[]? wkReplacementJudges;
    private Dictionary<string, Kampfgericht>? wkJudgingTables;
    private string savePath;
    private string applicationFolder = System.AppContext.BaseDirectory;

    [GeneratedRegex(@"### Wettkampfname ###")]
    private static partial Regex WkNameRegex();
    
    [GeneratedRegex(@"### Datum ###")]
    private static partial Regex WkDateRegex();
    
    [GeneratedRegex(@"### Wettkampfort ###")]
    private static partial Regex WkPlaceRegex();
    
    [GeneratedRegex(@"### Uhrzeit ###")]
    private static partial Regex WkJudgesmeetingTimeRegex();
    
    [GeneratedRegex(@"### Kampfrichterverantwortlicher ###")]
    private static partial Regex WkResponsiblePersonRegex();

    public DocumentWriter(Storage marshalledStorage, string savePath)
    {
        this.wkName = marshalledStorage.wk_name;
        this.wkDate = marshalledStorage.wk_date;
        this.wkPlace = marshalledStorage.wk_place;
        this.wkResponsiblePerson = marshalledStorage.wk_responsible_person;
        this.wkJudgesMeetingTime = marshalledStorage.wk_judgesmeeting_time;
        this.wkReplacementJudges = marshalledStorage.wk_replacement_judges;
        this.wkJudgingTables = marshalledStorage.wk_judgingtables;
        this.savePath = savePath;
    }

    public ApplicationError Write()
    {
        try
        {
            CopyTemplateToPath();
            SetWkDataInDocument();
        }
        catch (Exception e)
        {
            Console.WriteLine(e.Message + " : " + e.Source + " : " + e.Data);
            return ApplicationError.CSharpWriteError;
        }
        
        return ApplicationError.NoError;
    }

    private void CopyTemplateToPath()
    { 
        File.Copy(Path.Join(this.applicationFolder, @"../Resources/Vorlage_Einsatzplan_Leer.docx"), this.savePath, true);
    }

    private void SetWkDataInDocument()
    {
        using (WordprocessingDocument document = WordprocessingDocument.Open(this.savePath, true))
        {
            string? documentText = null;
            
            if (document.MainDocumentPart is null)
            {
                throw new ArgumentNullException("MainDocumentPart of template file is null.");
            }
            using (StreamReader streamReader = new StreamReader(document.MainDocumentPart.GetStream()))
            {
                documentText = streamReader.ReadToEnd();
            }
            documentText = WkNameRegex().Replace(documentText, this.wkName ?? "N/A");
            documentText = WkDateRegex().Replace(documentText, this.wkDate ?? "N/A");
            documentText = WkPlaceRegex().Replace(documentText, this.wkPlace ?? "N/A");
            documentText = WkJudgesmeetingTimeRegex().Replace(documentText, this.wkJudgesMeetingTime ?? "N/A");
            documentText = WkResponsiblePersonRegex().Replace(documentText, this.wkResponsiblePerson ?? "N/A");

            using (StreamWriter streamWriter = new StreamWriter(document.MainDocumentPart.GetStream(FileMode.Create)))
            {
                streamWriter.Write(documentText);
            }
        }
    }
    
}