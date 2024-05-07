using System.Text.RegularExpressions;
using DocumentFormat.OpenXml;
using DocumentFormat.OpenXml.Packaging;
using DocumentFormat.OpenXml.Wordprocessing;

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
    private string applicationFolder = AppContext.BaseDirectory;

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
    
    [GeneratedRegex(@"### Ersatzkampfrichter ###")]
    private static partial Regex WkReplacementJudgesRegex();

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
            RemoveAltersklassenRow();
            TableHandler handler = new TableHandler(this.wkJudgingTables.Values.ToArray(), null);
            Table[] regularTables = handler.GenerateRegularTables();
            Table[] finalTables = handler.GenerateFinalTables();
            WriteTablesToDocument(regularTables, finalTables);
        }
        catch (Exception e)
        {
            Console.WriteLine(e);
            return ApplicationError.CSharpWriteError;
        }
        
        return ApplicationError.NoError;
    }

    private void CopyTemplateToPath()
    {
        #if Windows
            File.Copy(Path.Join(Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData), @"de.philippremy.dtb-kampfrichtereinsatzplaene\Resources\Vorlage_Einsatzplan_Leer.docx"), this.savePath, true);
        #elif MacOS
            File.Copy(Path.Join(Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData), @"de.philippremy.dtb-kampfrichtereinsatzplaene/Resources/Vorlage_Einsatzplan_Leer.docx"), this.savePath, true);
        #else
            File.Copy(Path.Join(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), @"de.philippremy.dtb-kampfrichtereinsatzplaene/Resources/Vorlage_Einsatzplan_Leer.docx"), this.savePath, true);
        #endif
    }

    private void WriteTablesToDocument(Table[] regularTables, Table[] finalTables)
    {
        using (WordprocessingDocument document = WordprocessingDocument.Open(this.savePath, true))
        {
            if (document.MainDocumentPart is null)
            {
                throw new ArgumentNullException("MainDocumentPart of template file is null.");
            }

            OpenXmlElement insertMark = document.MainDocumentPart.Document.Body!.Descendants<Paragraph>().First(p => p.InnerText == "### Kampfgerichte ###");

            int musicTablesWrittenToPage = 0;
            int regularTablesWrittenToPage = 0;
            bool firstPage = true;
            
            foreach (Table regularTable in regularTables)
            {
                if (IsMusicTable(regularTable))
                {
                    if (musicTablesWrittenToPage >= 2 || (musicTablesWrittenToPage == 1 && regularTablesWrittenToPage == 1 ) || regularTablesWrittenToPage == 3 || (firstPage && musicTablesWrittenToPage == 1))
                    {
                        insertMark = insertMark.InsertAfterSelf(CreatePageBreak());
                        firstPage = false;
                        insertMark = insertMark.InsertAfterSelf(regularTable);
                        musicTablesWrittenToPage = 1; 
                        regularTablesWrittenToPage = 0;
                    } else
                    {
                        insertMark = insertMark.InsertAfterSelf(regularTable);
                        musicTablesWrittenToPage++;
                    }
                } else
                {
                    if (musicTablesWrittenToPage >= 2 || (musicTablesWrittenToPage == 1 && regularTablesWrittenToPage == 1) || regularTablesWrittenToPage == 3 || (firstPage && regularTablesWrittenToPage == 2))
                    {
                        insertMark = insertMark.InsertAfterSelf(CreatePageBreak());
                        firstPage = false;
                        insertMark = insertMark.InsertAfterSelf(regularTable);
                        musicTablesWrittenToPage = 0; 
                        regularTablesWrittenToPage = 1;
                    } else
                    {
                        insertMark = insertMark.InsertAfterSelf(regularTable);
                        regularTablesWrittenToPage++;
                    }
                }
            }

            if (finalTables.Length != 0)
            { 
                // The last element will never be a page break we introduced using the foreach loop above, so always insert one
                // Reset the counter
                insertMark = insertMark.InsertAfterSelf(CreatePageBreak());
                musicTablesWrittenToPage = 0;
                regularTablesWrittenToPage = 0;
                
                // Insert the final tables
                foreach (Table finalTable in finalTables)
                {
                    if (IsMusicTable(finalTable))
                    {
                        if (musicTablesWrittenToPage >= 2 || (musicTablesWrittenToPage == 1 && regularTablesWrittenToPage == 1 ) || regularTablesWrittenToPage == 3 || (firstPage && musicTablesWrittenToPage == 1))
                        {
                            insertMark = insertMark.InsertAfterSelf(CreatePageBreak());
                            firstPage = false;
                            insertMark = insertMark.InsertAfterSelf(finalTable);
                            musicTablesWrittenToPage = 1; 
                            regularTablesWrittenToPage = 0;
                        } else
                        {
                            insertMark = insertMark.InsertAfterSelf(finalTable);
                            musicTablesWrittenToPage++;
                        }
                    } else
                    {
                        if (musicTablesWrittenToPage >= 2 || (musicTablesWrittenToPage == 1 && regularTablesWrittenToPage == 1) || regularTablesWrittenToPage == 3 || (firstPage && regularTablesWrittenToPage == 2))
                        {
                            insertMark = insertMark.InsertAfterSelf(CreatePageBreak());
                            firstPage = false;
                            insertMark = insertMark.InsertAfterSelf(finalTable);
                            musicTablesWrittenToPage = 0; 
                            regularTablesWrittenToPage = 1;
                        } else
                        {
                            insertMark = insertMark.InsertAfterSelf(finalTable);
                            regularTablesWrittenToPage++;
                        }
                    }
                }
            }
            
            // Remove the initial insertion mark
            document.MainDocumentPart.Document.Body!.Descendants<Paragraph>().First(p => p.InnerText == "### Kampfgerichte ###").Remove();
            
            // Save the document if possible
            if (document.CanSave)
            {
                document.Save();
            }
        }
    }

    private Paragraph CreatePageBreak()
    {
        return new Paragraph(new Run(new Break() { Type = BreakValues.Page }));
    }

    private bool IsMusicTable(Table tableToCheck)
    {
        int noOfRows = tableToCheck.Elements<TableRow>().Count();
        return noOfRows > 12;
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
            
            // Create the string for the replacement judges
            string replacementJudgesString = "";
            if (this.wkReplacementJudges is not null && this.wkReplacementJudges.Length != 0)
            {
                for (int i = 0; i < this.wkReplacementJudges.Length; i++)
                {
                    if (i == this.wkReplacementJudges.Length - 1)
                    {
                        replacementJudgesString += this.wkReplacementJudges[i];
                    } else
                    {
                        replacementJudgesString += this.wkReplacementJudges[i] + ", ";
                    }
                }
            } else
            {
                replacementJudgesString = "Keine";
            }

            documentText = WkReplacementJudgesRegex().Replace(documentText, replacementJudgesString);

            using (StreamWriter streamWriter = new StreamWriter(document.MainDocumentPart.GetStream(FileMode.Create)))
            {
                streamWriter.Write(documentText);
            }
            
            // Save the document
            if (document.CanSave)
            {
                document.Save();
            }
        }
    }
    
    // TEMP: Remove row where Altersklassen will be specified in the future!
    private void RemoveAltersklassenRow()
    {
        using (WordprocessingDocument document = WordprocessingDocument.Open(this.savePath, true))
        {
            if (document.MainDocumentPart is null)
            {
                throw new ArgumentNullException("MainDocumentPart of template file is null.");
            }
            if (document.MainDocumentPart.Document.Body is null)
            {
                throw new ArgumentNullException("Body of template file is null.");
            }
            var tables = document.MainDocumentPart.Document.Descendants<Table>().ToList();
            List<TableCell> cellList = new List<TableCell>();
            foreach (Table t in tables)
            {
                var rows = t.Elements<TableRow>();
                foreach (TableRow row in rows)
                {
                    var cells = row.Elements<TableCell>();
                    foreach (TableCell cell in cells) 
                        cellList.Add(cell);
                }
            }
            var q = from c in cellList where c.InnerText == "### Altersklassen ###" select c.Parent;
            q.First().Remove();
            if (document.CanSave)
            {
                document.Save();
            }
        }
    }
    
}

public class TableHandler
{
    
    // Member variables
    private Kampfgericht[] m_kampfgerichte;
    private string[]? m_replacementJudges;
    private List<Kampfgericht> m_final_tables;
    private List<Kampfgericht> m_regular_tables;
    
    #if Windows
        private string m_pathToTableTemplate = Path.Join(Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData), @"de.philippremy.dtb-kampfrichtereinsatzplaene\Resources\Tabelle_Vorlage_Leer.docx");
    #elif MacOS
        private string m_pathToTableTemplate = Path.Join(Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData), @"de.philippremy.dtb-kampfrichtereinsatzplaene/Resources/Tabelle_Vorlage_Leer.docx");
    #else
        private string m_pathToTableTemplate = Path.Join(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), @"de.philippremy.dtb-kampfrichtereinsatzplaene/Resources/Tabelle_Vorlage_Leer.docx");
    #endif
    
    public TableHandler(Kampfgericht[] kampfgerichte, string[]? replacementJudges)
    {
        this.m_kampfgerichte = kampfgerichte;
        this.m_replacementJudges = replacementJudges;
        this.m_regular_tables = new List<Kampfgericht>();
        this.m_final_tables = new List<Kampfgericht>();
        SortTables();
    }

    public Table[] GenerateRegularTables()
    {
        List<Table> tables = [];
        // We can manually iterate when we use a IEnumerator
        IEnumerator<Kampfgericht> enumerator = this.m_regular_tables.GetEnumerator();
        while (enumerator.MoveNext())
        {
            Kampfgericht kampfgericht1 = enumerator.Current;
            if (enumerator.MoveNext())
            {
                Kampfgericht kampfgericht2 = enumerator.Current;
                tables.Add(GenerateDoubleTable(kampfgericht1, kampfgericht2, false, false));
            } else
            {
                tables.Add(GenerateSingleTable(kampfgericht1, false));
            }
        }
        return tables.ToArray();
    }
    
    public Table[] GenerateFinalTables()
    {
        List<Table> tables = [];
        // We can manually iterate when we use a IEnumerator
        IEnumerator<Kampfgericht> enumerator = this.m_final_tables.GetEnumerator();
        while (enumerator.MoveNext())
        {
            Kampfgericht kampfgericht1 = enumerator.Current;
            if (enumerator.MoveNext())
            {
                Kampfgericht kampfgericht2 = enumerator.Current;
                tables.Add(GenerateDoubleTable(kampfgericht1, kampfgericht2, true, true));
            } else
            {
                tables.Add(GenerateSingleTable(kampfgericht1, true));
            }
        }
        return tables.ToArray();
    }

    private Table GenerateSingleTable(Kampfgericht kampfgericht, bool isFinal)
    {
        // We have to clone the object so Unix (and possibly Windows) does not get upset when we open the template file multiple times
        WordprocessingDocument document = WordprocessingDocument.Open(this.m_pathToTableTemplate, true);
        Table table = document.MainDocumentPart.Document.Body.Descendants<Table>().First().Clone() as Table;
        document.Dispose();
        var cells = table.Descendants<TableCell>();
        foreach (var cell in cells)
        {
            switch (cell.InnerText)
            {
                case "### Name 1 ###":
                {
                    cell.Elements<Paragraph>().First().Elements<Run>().First().RemoveAllChildren<Text>();
                    if (isFinal)
                    {
                        cell.Elements<Paragraph>().First().Elements<Run>().First().AppendChild(new Text(kampfgericht.table_name != null ? "(Finale) " + kampfgericht.table_name : "N/A"));
                    }
                    else
                    {
                        cell.Elements<Paragraph>().First().Elements<Run>().First().AppendChild(new Text(kampfgericht.table_name ?? "N/A"));
                    }

                    break;
                }
                case "### Disziplin 1 ###":
                    cell.Elements<Paragraph>().First().Elements<Run>().First().RemoveAllChildren<Text>();
                    cell.Elements<Paragraph>().First().Elements<Run>().First().AppendChild(new Text(kampfgericht.table_kind ?? "N/A"));
                    break;
                case "### OK1.1 Name ###":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    cell.Elements<Paragraph>().First().AppendChild(new Run(new Text((kampfgericht.judges != null && kampfgericht.judges.ContainsKey("ok") ? kampfgericht.judges?["ok"].name : "") ?? "")));
                    break;
                case "### SK1.1 Name ###":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    cell.Elements<Paragraph>().First().AppendChild(new Run(new Text((kampfgericht.judges != null && kampfgericht.judges.ContainsKey("sk1") ? kampfgericht.judges?["sk1"].name : "") ?? "")));
                    break;
                case "### SK1.2 Name ###":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    cell.Elements<Paragraph>().First().AppendChild(new Run(new Text((kampfgericht.judges != null && kampfgericht.judges.ContainsKey("sk2") ? kampfgericht.judges?["sk2"].name : "") ?? "")));
                    break;
                case "### AK1.1 Name ###":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    cell.Elements<Paragraph>().First().AppendChild(new Run(new Text((kampfgericht.judges != null && kampfgericht.judges.ContainsKey("ak1") ? kampfgericht.judges?["ak1"].name : "") ?? "")));
                    break;
                case "### AK1.2 Name ###":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    cell.Elements<Paragraph>().First().AppendChild(new Run(new Text((kampfgericht.judges != null && kampfgericht.judges.ContainsKey("ak2") ? kampfgericht.judges?["ak2"].name : "") ?? "")));
                    break;
                case "### AK1.3 Name ###":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    cell.Elements<Paragraph>().First().AppendChild(new Run(new Text((kampfgericht.judges != null && kampfgericht.judges.ContainsKey("ak3") ? kampfgericht.judges?["ak3"].name : "") ?? "")));
                    break;
                case "### AK1.4 Name ###":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    cell.Elements<Paragraph>().First().AppendChild(new Run(new Text((kampfgericht.judges != null && kampfgericht.judges.ContainsKey("ak4") ? kampfgericht.judges?["ak4"].name : "") ?? "")));
                    break;
                case "### AIK1.1 Name ###":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    cell.Elements<Paragraph>().First().AppendChild(new Run(new Text((kampfgericht.judges != null && kampfgericht.judges.ContainsKey("aik1") ? kampfgericht.judges?["aik1"].name : "") ?? "")));
                    break;
                case "### AIK1.2 Name ###":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    cell.Elements<Paragraph>().First().AppendChild(new Run(new Text((kampfgericht.judges != null && kampfgericht.judges.ContainsKey("aik2") ? kampfgericht.judges?["aik2"].name : "") ?? "")));
                    break;
                case "### AIK1.3 Name ###":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    cell.Elements<Paragraph>().First().AppendChild(new Run(new Text((kampfgericht.judges != null && kampfgericht.judges.ContainsKey("aik3") ? kampfgericht.judges?["aik3"].name : "") ?? "")));
                    break;
                case "### AIK1.4 Name ###":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    cell.Elements<Paragraph>().First().AppendChild(new Run(new Text((kampfgericht.judges != null && kampfgericht.judges.ContainsKey("aik4") ? kampfgericht.judges?["aik4"].name : "") ?? "")));
                    break;
                // Now for the roles!
                case "OK1:":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    cell.Elements<Paragraph>().First().AppendChild(new Run(new Text("OK:")));
                    break;
                case "SK1.1:":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    cell.Elements<Paragraph>().First().AppendChild(new Run(new Text("SK1:")));
                    break;
                case "SK1.2:":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    cell.Elements<Paragraph>().First().AppendChild(new Run(new Text("SK2:")));
                    break;
                case "AK1.1:":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    cell.Elements<Paragraph>().First().AppendChild(new Run(new Text("AK1:")));
                    break;
                case "AK1.2:":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    cell.Elements<Paragraph>().First().AppendChild(new Run(new Text("AK2:")));
                    break;
                case "AK1.3:":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    cell.Elements<Paragraph>().First().AppendChild(new Run(new Text("AK3:")));
                    break;
                case "AK1.4:":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    cell.Elements<Paragraph>().First().AppendChild(new Run(new Text("AK4:")));
                    break;
                case "AIK1.1:":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    cell.Elements<Paragraph>().First().AppendChild(new Run(new Text("AIK1:")));
                    break;
                case "AIK1.2:":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    cell.Elements<Paragraph>().First().AppendChild(new Run(new Text("AIK2:")));
                    break;
                case "AIK1.3:":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    cell.Elements<Paragraph>().First().AppendChild(new Run(new Text("AIK3:")));
                    break;
                case "AIK1.4:":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    cell.Elements<Paragraph>().First().AppendChild(new Run(new Text("AIK4:")));
                    break;
                // Here we can null all fields of the second Kampfgericht. We don't have one...
                case "### Name 2 ###":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    cell.Elements<Paragraph>().First().AppendChild(new Run(new Text("")));
                    break;
                case "### Disziplin 2 ###":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    cell.Elements<Paragraph>().First().AppendChild(new Run(new Text("")));
                    break;
                case "### OK2.1 Name ###":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    cell.Elements<Paragraph>().First().AppendChild(new Run(new Text("")));
                    break;
                case "### SK2.1 Name ###":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    cell.Elements<Paragraph>().First().AppendChild(new Run(new Text("")));
                    break;
                case "### SK2.2 Name ###":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    cell.Elements<Paragraph>().First().AppendChild(new Run(new Text("")));
                    break;
                case "### AK2.1 Name ###":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    cell.Elements<Paragraph>().First().AppendChild(new Run(new Text("")));
                    break;
                case "### AK2.2 Name ###":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    cell.Elements<Paragraph>().First().AppendChild(new Run(new Text("")));
                    break;
                case "### AK2.3 Name ###":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    cell.Elements<Paragraph>().First().AppendChild(new Run(new Text("")));
                    break;
                case "### AK2.4 Name ###":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    cell.Elements<Paragraph>().First().AppendChild(new Run(new Text("")));
                    break;
                case "### AIK2.1 Name ###":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    cell.Elements<Paragraph>().First().AppendChild(new Run(new Text("")));
                    break;
                case "### AIK2.2 Name ###":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    cell.Elements<Paragraph>().First().AppendChild(new Run(new Text("")));
                    break;
                case "### AIK2.3 Name ###":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    cell.Elements<Paragraph>().First().AppendChild(new Run(new Text("")));
                    break;
                case "### AIK2.4 Name ###":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    cell.Elements<Paragraph>().First().AppendChild(new Run(new Text("")));
                    break;
                // Now for the roles!
                case "OK2:":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    cell.Elements<Paragraph>().First().AppendChild(new Run(new Text("")));
                    break;
                case "SK2.1:":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    cell.Elements<Paragraph>().First().AppendChild(new Run(new Text("")));
                    break;
                case "SK2.2:":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    cell.Elements<Paragraph>().First().AppendChild(new Run(new Text("")));
                    break;
                case "AK2.1:":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    cell.Elements<Paragraph>().First().AppendChild(new Run(new Text("")));
                    break;
                case "AK2.2:":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    cell.Elements<Paragraph>().First().AppendChild(new Run(new Text("")));
                    break;
                case "AK2.3:":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    cell.Elements<Paragraph>().First().AppendChild(new Run(new Text("")));
                    break;
                case "AK2.4:":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    cell.Elements<Paragraph>().First().AppendChild(new Run(new Text("")));
                    break;
                case "AIK2.1:":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    cell.Elements<Paragraph>().First().AppendChild(new Run(new Text("")));
                    break;
                case "AIK2.2:":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    cell.Elements<Paragraph>().First().AppendChild(new Run(new Text("")));
                    break;
                case "AIK2.3:":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    cell.Elements<Paragraph>().First().AppendChild(new Run(new Text("")));
                    break;
                case "AIK2.4:":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    cell.Elements<Paragraph>().First().AppendChild(new Run(new Text("")));
                    break;
            }
        }
        
        // We can safely assume that the last four rows can be deleted, if this is not a Kampfgericht mit Geradeturnen auf Musik as type
        if (kampfgericht.table_kind != "Geradeturnen auf Musik")
        {
            var rows = table.Descendants<TableRow>().ToList();
            // There is a spacing row in between, we don't want this removed!
            rows[^2].Remove();
            rows[^3].Remove();
            rows[^4].Remove();
            rows[^5].Remove();
        }
        
        return table;
    }
    
    private Table GenerateDoubleTable(Kampfgericht kampfgericht, Kampfgericht kampfgericht2, bool isFinal, bool isFinal2)
    {
        // We have to clone the object so Unix (and possibly Windows) does not get upset when we open the template file multiple times
        WordprocessingDocument document = WordprocessingDocument.Open(this.m_pathToTableTemplate, true);
        Table table = document.MainDocumentPart.Document.Body.Descendants<Table>().First().Clone() as Table;
        document.Dispose();
        var cells = table.Descendants<TableCell>();
        foreach (var cell in cells)
        {
            switch (cell.InnerText)
            {
                case "### Name 1 ###":
                {
                    cell.Elements<Paragraph>().First().Elements<Run>().First().RemoveAllChildren<Text>();
                    if (isFinal)
                    {
                        cell.Elements<Paragraph>().First().Elements<Run>().First().AppendChild(new Text(kampfgericht.table_name != null ? "(Finale) " + kampfgericht.table_name : "N/A"));
                    }
                    else
                    {
                        cell.Elements<Paragraph>().First().Elements<Run>().First().AppendChild(new Text(kampfgericht.table_name ?? "N/A"));
                    }

                    break;
                }
                case "### Disziplin 1 ###":
                    cell.Elements<Paragraph>().First().Elements<Run>().First().RemoveAllChildren<Text>();
                    cell.Elements<Paragraph>().First().Elements<Run>().First().AppendChild(new Text(kampfgericht.table_kind ?? "N/A"));
                    break;
                case "### OK1.1 Name ###":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    cell.Elements<Paragraph>().First().AppendChild(new Run(new Text((kampfgericht.judges != null && kampfgericht.judges.ContainsKey("ok") ? kampfgericht.judges?["ok"].name : "") ?? "")));
                    break;
                case "### SK1.1 Name ###":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    cell.Elements<Paragraph>().First().AppendChild(new Run(new Text((kampfgericht.judges != null && kampfgericht.judges.ContainsKey("sk1") ? kampfgericht.judges?["sk1"].name : "") ?? "")));
                    break;
                case "### SK1.2 Name ###":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    cell.Elements<Paragraph>().First().AppendChild(new Run(new Text((kampfgericht.judges != null && kampfgericht.judges.ContainsKey("sk2") ? kampfgericht.judges?["sk2"].name : "") ?? "")));
                    break;
                case "### AK1.1 Name ###":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    cell.Elements<Paragraph>().First().AppendChild(new Run(new Text((kampfgericht.judges != null && kampfgericht.judges.ContainsKey("ak1") ? kampfgericht.judges?["ak1"].name : "") ?? "")));
                    break;
                case "### AK1.2 Name ###":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    cell.Elements<Paragraph>().First().AppendChild(new Run(new Text((kampfgericht.judges != null && kampfgericht.judges.ContainsKey("ak2") ? kampfgericht.judges?["ak2"].name : "") ?? "")));
                    break;
                case "### AK1.3 Name ###":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    cell.Elements<Paragraph>().First().AppendChild(new Run(new Text((kampfgericht.judges != null && kampfgericht.judges.ContainsKey("ak3") ? kampfgericht.judges?["ak3"].name : "") ?? "")));
                    break;
                case "### AK1.4 Name ###":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    cell.Elements<Paragraph>().First().AppendChild(new Run(new Text((kampfgericht.judges != null && kampfgericht.judges.ContainsKey("ak4") ? kampfgericht.judges?["ak4"].name : "") ?? "")));
                    break;
                case "### AIK1.1 Name ###":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    cell.Elements<Paragraph>().First().AppendChild(new Run(new Text((kampfgericht.judges != null && kampfgericht.judges.ContainsKey("aik1") ? kampfgericht.judges?["aik1"].name : "") ?? "")));
                    break;
                case "### AIK1.2 Name ###":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    cell.Elements<Paragraph>().First().AppendChild(new Run(new Text((kampfgericht.judges != null && kampfgericht.judges.ContainsKey("aik2") ? kampfgericht.judges?["aik2"].name : "") ?? "")));
                    break;
                case "### AIK1.3 Name ###":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    cell.Elements<Paragraph>().First().AppendChild(new Run(new Text((kampfgericht.judges != null && kampfgericht.judges.ContainsKey("aik3") ? kampfgericht.judges?["aik3"].name : "") ?? "")));
                    break;
                case "### AIK1.4 Name ###":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    cell.Elements<Paragraph>().First().AppendChild(new Run(new Text((kampfgericht.judges != null && kampfgericht.judges.ContainsKey("aik4") ? kampfgericht.judges?["aik4"].name : "") ?? "")));
                    break;
                // Now for the roles!
                case "OK1:":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    cell.Elements<Paragraph>().First().AppendChild(new Run(new Text("OK:")));
                    break;
                case "SK1.1:":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    cell.Elements<Paragraph>().First().AppendChild(new Run(new Text("SK1:")));
                    break;
                case "SK1.2:":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    cell.Elements<Paragraph>().First().AppendChild(new Run(new Text("SK2:")));
                    break;
                case "AK1.1:":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    cell.Elements<Paragraph>().First().AppendChild(new Run(new Text("AK1:")));
                    break;
                case "AK1.2:":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    cell.Elements<Paragraph>().First().AppendChild(new Run(new Text("AK2:")));
                    break;
                case "AK1.3:":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    cell.Elements<Paragraph>().First().AppendChild(new Run(new Text("AK3:")));
                    break;
                case "AK1.4:":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    cell.Elements<Paragraph>().First().AppendChild(new Run(new Text("AK4:")));
                    break;
                // We need to be careful and only print these if we need to!
                case "AIK1.1:":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    if (kampfgericht.table_kind == "Geradeturnen auf Musik")
                    {
                        cell.Elements<Paragraph>().First().AppendChild(new Run(new Text("AIK1:")));
                    }
                    else
                    {
                        cell.Elements<Paragraph>().First().AppendChild(new Run(new Text("")));
                    }
                    break;
                case "AIK1.2:":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    if (kampfgericht.table_kind == "Geradeturnen auf Musik")
                    {
                        cell.Elements<Paragraph>().First().AppendChild(new Run(new Text("AIK2:")));
                    }
                    else
                    {
                        cell.Elements<Paragraph>().First().AppendChild(new Run(new Text("")));
                    }
                    break;
                case "AIK1.3:":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    if (kampfgericht.table_kind == "Geradeturnen auf Musik")
                    {
                        cell.Elements<Paragraph>().First().AppendChild(new Run(new Text("AIK3:")));
                    }
                    else
                    {
                        cell.Elements<Paragraph>().First().AppendChild(new Run(new Text("")));
                    }
                    break;
                case "AIK1.4:":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    if (kampfgericht.table_kind == "Geradeturnen auf Musik")
                    {
                        cell.Elements<Paragraph>().First().AppendChild(new Run(new Text("AIK4:")));
                    }
                    else
                    {
                        cell.Elements<Paragraph>().First().AppendChild(new Run(new Text("")));
                    }
                    break;
                // Create the other side!
                case "### Name 2 ###":
                {
                    cell.Elements<Paragraph>().First().Elements<Run>().First().RemoveAllChildren<Text>();
                    if (isFinal2)
                    {
                        cell.Elements<Paragraph>().First().Elements<Run>().First().AppendChild(new Text(kampfgericht2.table_name != null ? "(Finale) " + kampfgericht2.table_name : "N/A"));
                    }
                    else
                    {
                        cell.Elements<Paragraph>().First().Elements<Run>().First().AppendChild(new Text(kampfgericht2.table_name ?? "N/A"));
                    }

                    break;
                }
                case "### Disziplin 2 ###":
                    cell.Elements<Paragraph>().First().Elements<Run>().First().RemoveAllChildren<Text>();
                    cell.Elements<Paragraph>().First().Elements<Run>().First().AppendChild(new Text(kampfgericht2.table_kind ?? "N/A"));
                    break;
                case "### OK2.1 Name ###":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    cell.Elements<Paragraph>().First().AppendChild(new Run(new Text((kampfgericht2.judges != null && kampfgericht2.judges.ContainsKey("ok") ? kampfgericht2.judges?["ok"].name : "") ?? "")));
                    break;
                case "### SK2.1 Name ###":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    cell.Elements<Paragraph>().First().AppendChild(new Run(new Text((kampfgericht2.judges != null && kampfgericht2.judges.ContainsKey("sk1") ? kampfgericht2.judges?["sk1"].name : "") ?? "")));
                    break;
                case "### SK2.2 Name ###":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    cell.Elements<Paragraph>().First().AppendChild(new Run(new Text((kampfgericht2.judges != null && kampfgericht2.judges.ContainsKey("sk2") ? kampfgericht2.judges?["sk2"].name : "") ?? "")));
                    break;
                case "### AK2.1 Name ###":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    cell.Elements<Paragraph>().First().AppendChild(new Run(new Text((kampfgericht2.judges != null && kampfgericht2.judges.ContainsKey("ak1") ? kampfgericht2.judges?["ak1"].name : "") ?? "")));
                    break;
                case "### AK2.2 Name ###":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    cell.Elements<Paragraph>().First().AppendChild(new Run(new Text((kampfgericht2.judges != null && kampfgericht2.judges.ContainsKey("ak2") ? kampfgericht2.judges?["ak2"].name : "") ?? "")));
                    break;
                case "### AK2.3 Name ###":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    cell.Elements<Paragraph>().First().AppendChild(new Run(new Text((kampfgericht2.judges != null && kampfgericht2.judges.ContainsKey("ak3") ? kampfgericht2.judges?["ak3"].name : "") ?? "")));
                    break;
                case "### AK2.4 Name ###":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    cell.Elements<Paragraph>().First().AppendChild(new Run(new Text((kampfgericht2.judges != null && kampfgericht2.judges.ContainsKey("ak4") ? kampfgericht2.judges?["ak4"].name : "") ?? "")));
                    break;
                case "### AIK2.1 Name ###":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    cell.Elements<Paragraph>().First().AppendChild(new Run(new Text((kampfgericht2.judges != null && kampfgericht2.judges.ContainsKey("aik1") ? kampfgericht2.judges?["aik1"].name : "") ?? "")));
                    break;
                case "### AIK2.2 Name ###":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    cell.Elements<Paragraph>().First().AppendChild(new Run(new Text((kampfgericht2.judges != null && kampfgericht2.judges.ContainsKey("aik2") ? kampfgericht2.judges?["aik2"].name : "") ?? "")));
                    break;
                case "### AIK2.3 Name ###":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    cell.Elements<Paragraph>().First().AppendChild(new Run(new Text((kampfgericht2.judges != null && kampfgericht2.judges.ContainsKey("aik3") ? kampfgericht2.judges?["aik3"].name : "") ?? "")));
                    break;
                case "### AIK2.4 Name ###":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    cell.Elements<Paragraph>().First().AppendChild(new Run(new Text((kampfgericht2.judges != null && kampfgericht2.judges.ContainsKey("aik4") ? kampfgericht2.judges?["aik4"].name : "") ?? "")));
                    break;
                // Now for the roles!
                case "OK2:":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    cell.Elements<Paragraph>().First().AppendChild(new Run(new Text("OK:")));
                    break;
                case "SK2.1:":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    cell.Elements<Paragraph>().First().AppendChild(new Run(new Text("SK1:")));
                    break;
                case "SK2.2:":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    cell.Elements<Paragraph>().First().AppendChild(new Run(new Text("SK2:")));
                    break;
                case "AK2.1:":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    cell.Elements<Paragraph>().First().AppendChild(new Run(new Text("AK1:")));
                    break;
                case "AK2.2:":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    cell.Elements<Paragraph>().First().AppendChild(new Run(new Text("AK2:")));
                    break;
                case "AK2.3:":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    cell.Elements<Paragraph>().First().AppendChild(new Run(new Text("AK3:")));
                    break;
                case "AK2.4:":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    cell.Elements<Paragraph>().First().AppendChild(new Run(new Text("AK4:")));
                    break;
                // Same again here. Careful if we really need this.
                case "AIK2.1:":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    if (kampfgericht2.table_kind == "Geradeturnen auf Musik")
                    {
                        cell.Elements<Paragraph>().First().AppendChild(new Run(new Text("AIK1:")));
                    }
                    else
                    {
                        cell.Elements<Paragraph>().First().AppendChild(new Run(new Text("")));
                    }
                    break;
                case "AIK2.2:":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    if (kampfgericht2.table_kind == "Geradeturnen auf Musik")
                    {
                        cell.Elements<Paragraph>().First().AppendChild(new Run(new Text("AIK2:")));
                    }
                    else
                    {
                        cell.Elements<Paragraph>().First().AppendChild(new Run(new Text("")));
                    }
                    break;
                case "AIK2.3:":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    if (kampfgericht2.table_kind == "Geradeturnen auf Musik")
                    {
                        cell.Elements<Paragraph>().First().AppendChild(new Run(new Text("AIK3:")));
                    }
                    else
                    {
                        cell.Elements<Paragraph>().First().AppendChild(new Run(new Text("")));
                    }
                    break;
                case "AIK2.4:":
                    cell.Elements<Paragraph>().First().RemoveAllChildren<Run>();
                    if (kampfgericht2.table_kind == "Geradeturnen auf Musik")
                    {
                        cell.Elements<Paragraph>().First().AppendChild(new Run(new Text("AIK4:")));
                    }
                    else
                    {
                        cell.Elements<Paragraph>().First().AppendChild(new Run(new Text("")));
                    }
                    break;
            }
        }
        
        // We can safely assume that the last four rows can be deleted, if both are not a Kampfgericht mit Geradeturnen auf Musik as type
        if (kampfgericht.table_kind != "Geradeturnen auf Musik" && kampfgericht2.table_kind != "Geradeturnen auf Musik")
        {
            var rows = table.Descendants<TableRow>().ToList();
            // There is a spacing row in between, we don't want this removed!
            rows[^2].Remove();
            rows[^3].Remove();
            rows[^4].Remove();
            rows[^5].Remove();
        }
        
        return table;
    }

    private void SortTables()
    {
        // Differentiate between Final and non-final table
        foreach (Kampfgericht kampfgericht in this.m_kampfgerichte)
        {
            if (kampfgericht.table_is_finale.HasValue && kampfgericht.table_is_finale.Value)
            {
                this.m_final_tables.Add(kampfgericht);
            } else if (kampfgericht.table_is_finale.HasValue && !kampfgericht.table_is_finale.Value)
            {
                this.m_regular_tables.Add(kampfgericht);
            }
        }
        // Sort the tables alphabetically
        this.m_final_tables = this.m_final_tables.OrderBy(val => val.table_name).ToList();
        this.m_regular_tables = this.m_regular_tables.OrderBy(val => val.table_name).ToList();
    }
}