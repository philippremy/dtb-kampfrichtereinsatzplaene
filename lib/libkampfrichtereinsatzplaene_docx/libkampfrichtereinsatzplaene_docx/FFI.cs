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
            string? rawJSONData = Marshal.PtrToStringAuto(json_data);
            if (rawJSONData == null) return ApplicationError.MarshalJSONNullError;
            storage = JsonSerializer.Deserialize<Storage>(rawJSONData, SourceGenerationContextStorage.Default.Storage);
            
            savePath = Marshal.PtrToStringAuto(save_path);
            if (savePath == null) return ApplicationError.MarshalSavePathNullError;
        }
        catch (Exception e)
        {
            Console.WriteLine(e.Message);
            return e switch
            {
                ArgumentNullException => ApplicationError.DeserializeArgumentNullError,
                JsonException => ApplicationError.DeserializeJSONError,
                NotSupportedException => ApplicationError.DeserializeNotSupportedError,
                _ => ApplicationError.UnknownError
            };
        }

        if (storage == null) return ApplicationError.StorageNullError;

        DocumentWriter writer = new DocumentWriter(storage, savePath);
        
        return writer.Write();

    }
}