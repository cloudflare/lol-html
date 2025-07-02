use super::*;

impl_content_mutation_handlers! { doc_end: DocumentEnd [
    /// Inserts the content at the end of the document, either as raw text or as HTML.
    ///
    /// The content should be a valid UTF-8 string.
    ///
    /// Returns 0 if successful, and -1 otherwise. The actual error message
    /// can be obtained using the `lol_html_take_last_error` function.
    lol_html_doc_end_append => append,
] }
