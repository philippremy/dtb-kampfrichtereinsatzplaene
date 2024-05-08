using System.Collections;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using System.Text.Json;


namespace libkampfrichtereinsatzplaene_docx;

/// <summary>
/// The Foreign-Function-Interface Class (FFI) for Interoperability with Rust.
/// Contains all relevant Marshaling Functions and prepares the data for used in Managed .NET Types.
/// </summary>
public class FFI
{
    /// <summary>
    /// Keep this for testing purposes.
    /// </summary>
    /// <remarks>
    /// May be called by UnmanagedCallers.
    /// </remarks>
    [UnmanagedCallersOnly(EntryPoint = "stub_func")]
    public static void StubFunc()
    {
        Console.Out.WriteLine("This is currently a stub.");
    }

    /// <summary>
    /// The Main Entry Point which Rust has to call with the correct arguments.
    /// Marshalling magic happens here.
    /// Hands off to DocumentWriter afterwards.
    /// </summary>
    /// <remarks>
    /// May be called by UnmanagedCallers.
    /// </remarks>
    [UnmanagedCallersOnly(EntryPoint = "ffi_create_from_raw_data")]
    public static ApplicationError CreateFromRawData(IntPtr json_data, IntPtr save_path)
    {
        Storage? storage;
        string? savePath;
        
        try
        {
            string? rawJSONData = Marshal.PtrToStringUTF8(json_data);
            if (rawJSONData == null) { PrintError("Marshalled JSON data was null (likely an encoding error)."); return ApplicationError.MarshalJSONNullError; }
            storage = JsonSerializer.Deserialize<Storage>(rawJSONData, SourceGenerationContextStorage.Default.Storage);
            
            savePath = Marshal.PtrToStringUTF8(save_path);
            if (savePath == null) { PrintError("Marshalled SavePath raw data was null (likely an encoding error)."); return ApplicationError.MarshalSavePathNullError; }
        }
        catch (Exception e)
        {
            PrintErrorFromException(e);
            return e switch
            {
                ArgumentNullException => ApplicationError.DeserializeArgumentNullError,
                JsonException => ApplicationError.DeserializeJSONError,
                NotSupportedException => ApplicationError.DeserializeNotSupportedError,
                _ => ApplicationError.UnknownError
            };
        }

        if (storage == null) { PrintError("Storage from marshalled data was null."); return ApplicationError.StorageNullError; }

        DocumentWriter writer = new DocumentWriter(storage, savePath);
        
        return writer.Write();

    }
    
    private static ApplicationError CreateFromRawDataInternal(IntPtr json_data, string save_path)
    {
        Storage? storage;
        string? savePath = save_path;
        
        try
        {
            string? rawJSONData = Marshal.PtrToStringUTF8(json_data);
            if (rawJSONData == null) { PrintError("Marshalled JSON data was null (likely an encoding error)."); return ApplicationError.MarshalJSONNullError; }
            storage = JsonSerializer.Deserialize<Storage>(rawJSONData, SourceGenerationContextStorage.Default.Storage);
        }
        catch (Exception e)
        {
            PrintErrorFromException(e);
            return e switch
            {
                ArgumentNullException => ApplicationError.DeserializeArgumentNullError,
                JsonException => ApplicationError.DeserializeJSONError,
                NotSupportedException => ApplicationError.DeserializeNotSupportedError,
                _ => ApplicationError.UnknownError
            };
        }

        if (storage == null) { PrintError("Storage from marshalled data was null."); return ApplicationError.StorageNullError; }

        DocumentWriter writer = new DocumentWriter(storage, savePath);
        
        return writer.Write();

    }

    [UnmanagedCallersOnly(EntryPoint = "ffi_create_pdf_from_raw_data")]
    public static ApplicationError CreatePDFFromRawData(IntPtr json_data, IntPtr save_path)
    {
        string? savePath;

        try
        {
            savePath = Marshal.PtrToStringUTF8(save_path);
            if (savePath == null) { PrintError("Marshalled SavePath raw data was null (likely an encoding error)."); return ApplicationError.MarshalSavePathNullError; }
            FileInfo savePathInfo = new FileInfo(savePath);
            string docxSavePath = savePathInfo.FullName.Replace(".pdf", "_temp.docx");
            ApplicationError docxGeneratedCode = CreateFromRawDataInternal(json_data, docxSavePath);
            if (docxGeneratedCode != ApplicationError.NoError)
            {
                return docxGeneratedCode;
            }
            PDFWriter pdfWriter = new PDFWriter(savePath, docxSavePath);
            return pdfWriter.WriteToPDF();
        }
        catch (Exception e)
        {
            PrintErrorFromException(e);
            return ApplicationError.CSharpWriteError;
        }
    }

    public static void PrintError(string message, [CallerLineNumber] int sourceLineNumber = 0, [CallerMemberName] string memberName = "N/A", [CallerFilePath] string sourceFilePath = "N/A")
    {
        Console.Error.WriteLine("C# Error in File '" + Path.GetFileName(sourceFilePath) + "' on Line " + sourceLineNumber + " in Method '" + memberName + "':");
        Console.Error.WriteLine("Message: '" + message + "'\n");
    }
    
    public static void PrintLog(string message, [CallerLineNumber] int sourceLineNumber = 0, [CallerMemberName] string memberName = "N/A", [CallerFilePath] string sourceFilePath = "N/A")
    {
        
        Console.Out.WriteLine("C# Logging (Line " + sourceLineNumber + " in " + Path.GetFileName(sourceFilePath) + "):");
        Console.Out.WriteLine("Message: '" + message + "'\n");
    }

    public static void PrintErrorFromException(Exception e, [CallerFilePath] string sourceFilePath = "N/A")
    {
        string stackTrace = e.StackTrace ?? "Stacktrace unavailable";
        string failingMethod = e.TargetSite?.ToString() ?? "Method information unavailable.";
        string message = e.Message;
        string? data = null;
        foreach (DictionaryEntry entry in e.Data)
        {
            data += "[KEY]: " + (entry.Key.ToString() ?? "N/A") + "   :   " + (entry.Value?.ToString() ?? "N/A" + " :[VALUE]\n");
        }
        Console.Error.WriteLine("C# Exception thrown in File '" + Path.GetFileName(sourceFilePath) + " in Method '" + failingMethod + "':");
        Console.Error.WriteLine("Message: '" + message + "'\n");
        Console.Error.WriteLine("Data:\n\n" + (data ?? "No data available.") + "\n");
        Console.Error.WriteLine("Stack Trace:\n" + stackTrace + "\n");
    }
    
}