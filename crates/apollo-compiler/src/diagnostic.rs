use crate::FileId;
use crate::NodeLocation;
use crate::SourceMap;
use ariadne::ColorGenerator;
use ariadne::ReportKind;
use std::fmt;
use std::io;
use std::ops::Range;

type MappedSpan = (FileId, Range<usize>);

/// Translate a byte-offset location into a char-offset location for use with ariadne.
fn map_span(sources: &SourceMap, location: NodeLocation) -> Option<MappedSpan> {
    let source = sources.get(&location.file_id)?;
    let mapped_source = source.mapped_source();
    let start = mapped_source.map_index(location.offset());
    let end = mapped_source.map_index(location.end_offset());
    Some((location.file_id, start..end))
}

pub struct DiagnosticBuilder {
    sources: SourceMap,
    colors: ColorGenerator,
    report: ariadne::ReportBuilder<'static, MappedSpan>,
}

impl DiagnosticBuilder {
    pub fn new(sources: SourceMap, location: Option<NodeLocation>) -> Self {
        let (file_id, range) = location
            .and_then(|location| map_span(&sources, location))
            .unwrap_or((FileId::NONE, 0..0));
        Self {
            sources,
            colors: ColorGenerator::new(),
            report: ariadne::Report::build(ReportKind::Error, file_id, range.start),
        }
    }

    pub fn with_message(&mut self, message: impl ToString) {
        self.report.set_message(message);
    }

    pub fn with_help(&mut self, help: impl ToString) {
        self.report.set_help(help);
    }

    pub fn with_note(&mut self, note: impl ToString) {
        self.report.set_note(note);
    }

    pub fn with_label_opt(&mut self, location: Option<NodeLocation>, message: impl ToString) {
        if let Some(mapped_span) = location.and_then(|location| map_span(&self.sources, location)) {
            self.report.add_label(
                ariadne::Label::new(mapped_span)
                    .with_message(message)
                    .with_color(self.colors.next()),
            );
        }
    }

    pub fn with_color(self, color: bool) -> Self {
        let Self {
            sources,
            colors,
            report,
        } = self;
        let report = report.with_config(ariadne::Config::default().with_color(color));
        Self {
            sources,
            colors,
            report,
        }
    }

    pub fn finish(self) -> DiagnosticReport {
        DiagnosticReport {
            sources: self.sources,
            report: self.report.finish(),
        }
    }
}

pub struct DiagnosticReport {
    sources: SourceMap,
    report: ariadne::Report<'static, MappedSpan>,
}

impl DiagnosticReport {
    pub fn builder(sources: SourceMap, location: Option<NodeLocation>) -> DiagnosticBuilder {
        DiagnosticBuilder::new(sources, location)
    }
}

impl fmt::Display for DiagnosticReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        struct Adaptor<'a, 'b>(&'a mut fmt::Formatter<'b>);

        impl io::Write for Adaptor<'_, '_> {
            fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
                let s = std::str::from_utf8(buf).map_err(|_| io::ErrorKind::Other)?;
                self.0.write_str(s).map_err(|_| io::ErrorKind::Other)?;
                Ok(buf.len())
            }

            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }

        self.report
            // .report(self.sources, color)
            .write(&self.sources, Adaptor(f))
            .map_err(|_| fmt::Error)
    }
}
