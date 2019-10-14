use regex::Regex;
use std::collections::BTreeMap;
use std::iter::FromIterator;
use std::io::{self, BufRead, Write};

mod pane;
use pane::Surface;

fn main() -> io::Result<()> {
    let mut gh = LineGH::new();

    let command_prefix = b">>> ";
    let stdin = io::stdin();
    let stdout = io::stdout();
    loop {
        stdout.lock().write_all(command_prefix)?;
        stdout.lock().flush()?;

        let mut stdin = stdin.lock();
        let buffer = stdin.fill_buf()?;
        if buffer.is_empty() {
            return Ok(());
        }

        let lines = buffer
            .split(u8::is_ascii_control)
            .map(std::str::from_utf8)
            .flatten()
            .filter(|s| !s.is_empty())
            .collect::<Vec<&str>>();

        for line in lines {
            let command = parse_command(&line);
            handle_command(&mut stdout.lock(), &mut gh, command)?;
        }

        let len = buffer.len();
        stdin.consume(len);
    }
}

#[derive(Debug)]
enum Command {
    Print,
    SetGHType,
    SetGap(usize),
    SetConnectionSize(usize),
    Structure,
    AddEdge(Box<String>),
    ConnectEdges(usize, usize),
}

fn parse_command(line: &str) -> Option<Command> {
    let clean_line = line.trim();

    if clean_line.starts_with("print") {
        Some(Command::Print)
    } else if clean_line.starts_with("structure") {
        Some(Command::Structure)
    } else if clean_line.starts_with("settings") {
        let gap_regex = Regex::new(r"settings gap edge (?P<size>.+)").unwrap();
        let connection_size_regex = Regex::new(r"settings gap vert (?P<size>.+)").unwrap();

        if gap_regex.is_match(clean_line) {
            let caps = gap_regex.captures(clean_line).unwrap();
            let size = caps["size"].parse().unwrap();

            Some(Command::SetGap(size))
        } else if connection_size_regex.is_match(clean_line) {
            let caps = connection_size_regex.captures(clean_line).unwrap();
            let size = caps["size"].parse().unwrap();

            Some(Command::SetConnectionSize(size))
        } else if clean_line.contains("settings related") {
            Some(Command::SetGHType)
        } else {
            None
        }
    }else {
        let add_edge_command = Regex::new(r"edge add (?P<data>.+)").unwrap();
        let add_verticale_command =
            Regex::new(r"edge connect (?P<first>\d+) (?P<second>\d+)").unwrap();

        if add_edge_command.is_match(clean_line) {
            let caps = add_edge_command.captures(clean_line).unwrap();
            Some(Command::AddEdge(Box::new(String::from(&caps["data"]))))
        } else if add_verticale_command.is_match(clean_line) {
            let caps = add_verticale_command.captures(clean_line).unwrap();
            let first = caps["first"].parse().unwrap();
            let second = caps["second"].parse().unwrap();

            Some(Command::ConnectEdges(first, second))
        } else {
            None
        }
    }
}

fn handle_command<W: Write>(
    w: &mut W,
    gh: &mut LineGH,
    command: Option<Command>,
) -> io::Result<()> {
    match command {
        Some(Command::Print) => {
            writeln!(w, "{}", gh)?;
        },
        Some(Command::Structure) => {},
        Some(Command::AddEdge(data)) => {
            gh.add_edge(&data);
        },
        Some(Command::ConnectEdges(from, to)) => {
            gh.connect(from, to);
        },
        Some(Command::SetGap(size)) => { gh.pane_settings.gap_size = size },
        Some(Command::SetConnectionSize(size)) => { gh.pane_settings.connection_size = size },
        Some(Command::SetGHType) => { 
            if gh.pane_settings.connection_type == pane::ConnectorType::General {
                gh.pane_settings.connection_type = pane::ConnectorType::Arrow;
            } else {
                gh.pane_settings.connection_type = pane::ConnectorType::General;
            }
        },
        None => {
            writeln!(w, "cannot hold this type of command")?;
        },
    }

    Ok(())
}

struct LineGH {
    // might use here real graph?
    vertices: BTreeMap<usize, Vec<usize>>,
    edges: Vec<String>,
    pane_settings: pane::PaneSettings,
}

impl LineGH {
    pub fn new() -> Self {
        LineGH::new_with_settings(pane::PaneSettings{
            gap_size: 1,
            connection_size: 1,
            connection_type: pane::ConnectorType::General,
        })
    }

    pub fn new_with_settings(settings: pane::PaneSettings) -> Self {
        LineGH {
            edges: Vec::new(),
            vertices: BTreeMap::new(),
            pane_settings: settings,
        }
    }

    pub fn add_edge(&mut self, edge: &str) -> usize {
        self.edges.push(String::from(edge));
        self.edges.len() - 1
    }

    pub fn connect(&mut self, e1: usize, e2: usize) {
        self.vertices.entry(e1).or_insert_with(Vec::new).push(e2);
        self.vertices.entry(e2).or_insert_with(Vec::new);
    }

    pub fn count_by(&self, i: usize) -> usize {
        match self.vertices.get(&i) {
            Some(connected_edges) => {
                connected_edges.len()
                    + self
                        .vertices
                        .values()
                        .fold(0, |acc, ver| acc + ver.iter().filter(|&&v| v == i).count())
            }
            None => 0,
        }
    }

    pub fn structure(&self) -> BTreeMap<usize, (usize, usize)> {
        BTreeMap::from_iter(
            self.vertices.keys().cloned().
                zip(self.vertices.iter().
                map(|(i, connected)| (connected.len(), self.count_by(*i) - connected.len())))
        )
    }
} 

impl std::fmt::Display for LineGH {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        //TODO: logic with boxes should be refactored
        let boxes = self
            .edges
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let count_connected = self.count_by(i);
                let single_box = FormatBox::new(s, 1);
                let max_on_line = f64::ceil(single_box.line_lenght() as f64 / self.pane_settings.connection_size as f64) as usize;
                if count_connected > max_on_line {
                    FormatBox::new(s, (count_connected - max_on_line) * self.pane_settings.connection_size + 1)
                } else {
                    single_box
                }
            })
            .collect::<Vec<FormatBox>>();

        let boxes_length = boxes
            .iter()
            .map(FormatBox::line_lenght)
            .collect::<Vec<usize>>();
        let mut pane = pane::ConnectedPane::new(&boxes_length, self.pane_settings.clone());

        for (node, friends) in &self.vertices {
            for friend in friends {
                pane.connect(*node, *friend);
            }
        }

        writeln!(f, "{}", pane.pane())?;

        let str_boxes = boxes.iter().map(String::from).collect::<Vec<String>>();
        let boxed_edges = flatten_line(
            &str_boxes.iter().map(|b| b.as_ref()).collect::<Vec<&str>>(),
            self.pane_settings.gap_size
        );
        write!(f, "{}", boxed_edges)?;
        Ok(())
    }
}

fn new_line(index: usize, count_in: usize, count_out: usize) -> String {
    format!("{} - in {} out {}", index, count_in, count_out)
}

struct FormatBox<'a> {
    message: &'a str,
    tab_size: usize,
}

impl<'a> FormatBox<'a> {
    fn new(s: &'a str, tab_size: usize) -> Self {
        FormatBox {
            message: s,
            tab_size,
        }
    }

    fn line_lenght(&self) -> usize {
        2 + self.tab_size * 2 + size_biggest_line(&self.message)
    }
}

impl<'a> std::fmt::Display for FormatBox<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let horizontal_tab = " ".repeat(self.tab_size);
        let horizontal_line = "-".repeat(self.line_lenght());
        let vertical_space = format!("|{}|", " ".repeat(self.line_lenght() - 2));

        let max_len = size_biggest_line(&self.message);
        let content = self
            .message
            .lines()
            .map(|l| format!("|{}{: <3$}{}|", horizontal_tab, l, horizontal_tab, max_len))
            .collect::<Vec<String>>()
            .join("\n");

        let vertical_space_lined = if self.tab_size > 0 {
            format!("{}\n", vertical_space)
        } else {
            "".to_owned()
        };

        write!(
            f,
            "{}\n\
             {}\
             {}\n\
             {}\
             {}",
            horizontal_line, vertical_space_lined, content, vertical_space_lined, horizontal_line
        )?;
        Ok(())
    }
}

impl<'a> std::convert::From<&FormatBox<'a>> for String {
    fn from(b: &FormatBox<'a>) -> String {
        format!("{}", b)
    }
}

fn flatten_line(src: &[&str], gap_size: usize) -> String {
    let element_with_max_lines = src
        .iter()
        .max_by(|x, y| x.lines().count().cmp(&y.lines().count()));
    let max_lines = match element_with_max_lines {
        Some(element) => element.lines().count(),
        None => 0,
    };

    let mut lines = String::new();
    for line_index in 0..max_lines {
        for source in src {
            let element_lines = source.lines().collect::<Vec<&str>>();
            let max_line_size = size_biggest_line(source);
            let line = match element_lines.get(line_index) {
                Some(line) => format!("{: <1$}", line, max_line_size),
                None => " ".repeat(max_line_size),
            };

            lines.push_str(&line);
            lines.push_str(&" ".repeat(gap_size));
        }
        lines.push('\n');
    }

    String::from(lines.trim())
}

fn size_biggest_line(s: &str) -> usize {
    s.lines().fold(
        0,
        |max, item| {
            if item.len() > max {
                item.len()
            } else {
                max
            }
        },
    )
}
