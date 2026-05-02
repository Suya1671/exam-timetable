// TODO: investigate https://lib.rs/crates/typst-as-lib as an alternative to the custom (currently AI-generated) world
use entity::id::{ExamId, TimeslotId};
use typst::World;
use typst::diag::{FileError, FileResult};
use typst::foundations::{Bytes, Datetime};
use typst::syntax::{FileId, Source, VirtualPath};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, LibraryExt};
use typst_kit::fonts::{FontSearcher, FontSlot};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
/// Data to generate a timetable PDF
pub struct TimetableData {
    /// The name of the school
    pub school_name: String,
    /// The title of the timetable
    pub title: String,
    /// The grades to include in the timetable
    pub grades: Vec<u8>,
    /// The days that make up the timetable
    pub days: Vec<TimetableDay>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
/// A day in the timetable
pub struct TimetableDay {
    /// The date of the day, in ISO 8601 format
    pub date: String,
    /// The sessions that occur on this day
    pub sessions: Vec<TimetableSession>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
/// A session in the timetable
pub struct TimetableSession {
    /// The session number (currently either 1 or 2)
    pub session_number: u8,
    /// The timeslot ID of the session
    pub timeslot_id: TimeslotId,
    /// The exams that occur in this session
    pub exams: Vec<TimetableExamEntry>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
/// An exam entry in the timetable session
pub struct TimetableExamEntry {
    /// The session ID of the exam
    pub session_id: i32,
    /// The exam ID of the exam
    pub exam_id: ExamId,
    /// The grade of the exam
    pub grade: i32,
    /// The subject of the exam
    pub subject: String,
    /// Custom name for the exam. Replaces <Paper number> in the display
    pub exam_name: Option<String>,
    /// The start of the exam, formatted as "hh:mm"
    pub start_time: String,
    /// The end of the exam, formatted as "hh:mm"
    pub end_time: String,
    /// The exam paper number
    pub paper_number: i32,
    pub locked: bool,
}

/// AI-generated (Gemini).
pub struct TimetableWorld {
    library: LazyHash<Library>,
    book: LazyHash<FontBook>,
    fonts: Vec<FontSlot>,
    source: Source,
    data: Bytes,
}

impl TimetableWorld {
    /// AI-generated (Gemini).
    pub fn new(template: String, data_json: String) -> Self {
        let fonts = FontSearcher::new().include_system_fonts(true).search();

        Self {
            library: LazyHash::new(Library::builder().build()),
            book: LazyHash::new(fonts.book),
            fonts: fonts.fonts,
            source: Source::new(FileId::new_fake(VirtualPath::new("/main.typ")), template),
            data: Bytes::new(data_json.into_bytes()),
        }
    }
}

impl World for TimetableWorld {
    /// AI-generated (Gemini).
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }

    /// AI-generated (Gemini).
    fn book(&self) -> &LazyHash<FontBook> {
        &self.book
    }

    /// AI-generated (Gemini).
    fn main(&self) -> FileId {
        self.source.id()
    }

    /// AI-generated (Gemini).
    fn source(&self, id: FileId) -> FileResult<Source> {
        if id == self.source.id() {
            Ok(self.source.clone())
        } else {
            Err(FileError::NotFound(id.vpath().as_rootless_path().into()))
        }
    }

    /// AI-generated (Gemini).
    fn file(&self, id: FileId) -> FileResult<Bytes> {
        if id.vpath().as_rootless_path().to_str() == Some("data.json") {
            Ok(self.data.clone())
        } else {
            Err(FileError::NotFound(id.vpath().as_rootless_path().into()))
        }
    }

    /// AI-generated (Gemini).
    fn font(&self, index: usize) -> Option<Font> {
        self.fonts.get(index).and_then(|slot| slot.get())
    }

    /// AI-generated (Gemini).
    fn today(&self, _offset: Option<i64>) -> Option<Datetime> {
        None
    }
}

/// AI-generated (Gemini).
pub fn render_pdf(data: &TimetableData) -> Result<Vec<u8>, String> {
    let template = include_str!("timetable_template.typ").to_string();
    let data_json = serde_json::to_string(data).map_err(|e| e.to_string())?;

    let world = TimetableWorld::new(template, data_json.clone());

    let document = typst::compile(&world).output.map_err(|errs| {
        errs.iter()
            .map(|e| e.message.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    })?;

    let pdf = typst_pdf::pdf(&document, &typst_pdf::PdfOptions::default()).map_err(|e| {
        e.iter()
            .map(|diag| diag.message.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    })?;

    Ok(pdf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_pdf_successfully() {
        let data_json = include_str!("data.json");
        let data: TimetableData = serde_json::from_str(data_json).expect("valid JSON");
        let result = render_pdf(&data);
        assert!(result.is_ok(), "render failed: {:?}", result.err());
        let pdf = result.unwrap();
        assert!(!pdf.is_empty(), "PDF is empty");
        assert!(
            pdf.starts_with(b"%PDF"),
            "PDF does not have PDF magic bytes"
        );
    }
}
