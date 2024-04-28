using System;
using System.Runtime.CompilerServices;
using System.Text.Json.Serialization;

namespace libkampfrichtereinsatzplaene_docx;

public class Kampfrichter
{
    public string? role { get; set; }
    public string? name { get; set; }
    public bool? doubleFound { get; set; }
}

public class Kampfgericht
{
    public string? uniqueID { get; set; }
    public string? table_name { get; set; }
    public string? table_kind { get; set; }
    public bool? table_is_finale { get; set; }
    public Dictionary<string, Kampfrichter>? judges { get; set; }
}

public class Storage
{
    public string? wk_name { get; set; }
    public string? wk_date { get; set; }
    public string? wk_place { get; set; }
    public string? wk_responsible_person { get; set; }
    public string? wk_judgesmeeting_time { get; set; }
    public string[]? wk_replacement_judges { get; set; }
    public Dictionary<string, Kampfgericht>? wk_judgingtables { get; set; }
}

[JsonSourceGenerationOptions(WriteIndented = true)]
[JsonSerializable(typeof(Storage))]
internal partial class SourceGenerationContextStorage : JsonSerializerContext
{
}

[JsonSerializable(typeof(Kampfgericht))]
internal partial class SourceGenerationContextKampfgericht : JsonSerializerContext
{
}

[JsonSerializable(typeof(Kampfrichter))]
internal partial class SourceGenerationContextKampfrichter : JsonSerializerContext
{
}

public enum ApplicationError {
    UnknownError = -1,
    NoError = 0,
    MutexPoisonedError = 1,
    JSONSerializeError = 2,
    CStringNullError = 3,
    MarshalJSONNullError = 4,
    DeserializeArgumentNullError = 5,
    DeserializeJSONError = 6,
    DeserializeNotSupportedError = 7,
    TauriWindowCreationError = 8,
    TauriWindowShowError = 9,
    RustWriteFileError = 10,
    MarshalSavePathNullError = 11,
    StorageNullError = 12,
    CSharpWriteError = 13,
    JSONDeserializeImporterError = 14,
    FailedToCreateStdOutFileError = 15,
    FailedToCreateStdErrFileError = 16,
    LibcDup2StdOutError = 17,
    LibcDup2StdErrError = 18,
}