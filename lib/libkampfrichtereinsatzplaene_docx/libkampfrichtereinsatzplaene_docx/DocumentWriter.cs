namespace libkampfrichtereinsatzplaene_docx;

public class DocumentWriter
{

    private string wkName;
    private string wkDate;
    private string wkPlace;
    private string wkResponsiblePerson;
    private string wkJudgesMeetingTime;
    private string[] wkReplacementJudges;
    private Kampfgericht[] wkJudgingTables;
    
    public DocumentWriter(Storage marshalledStorage, string savePath)
    {
        this.wkName = marshalledStorage.wk_name;
        this.wkDate = marshalledStorage.wk_date;
        this.wkPlace = marshalledStorage.wk_place;
        this.wkResponsiblePerson = marshalledStorage.wk_responsible_person;
        this.wkJudgesMeetingTime = marshalledStorage.wk_judgesmeeting_time;
        this.wkReplacementJudges = marshalledStorage.wk_replacement_judges;
        this.wkJudgingTables = marshalledStorage.wk_judgingtables;
    }

    public ApplicationError Write()
    {
        
        return ApplicationError.NoError;
    }
    
}