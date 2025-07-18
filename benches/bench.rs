use criterion::{criterion_group, criterion_main};
use glob::glob;
use std::fmt::{self, Debug};
use std::fs::File;
use std::io::Read;
use std::sync::LazyLock;

const CHUNK_SIZE: usize = 1024;

struct Input {
    pub name: String,
    pub length: usize,
    pub chunks: Vec<Vec<u8>>,
}

impl Debug for Input {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

static INPUTS: LazyLock<Vec<Input>> = LazyLock::new(|| {
    glob("benches/data/*.html")
        .unwrap()
        .map(|path| {
            let mut data = String::new();
            let path = path.unwrap();

            File::open(&path)
                .unwrap()
                .read_to_string(&mut data)
                .unwrap();

            let data = data.into_bytes();

            Input {
                name: path.file_name().unwrap().to_string_lossy().to_string(),
                length: data.len(),
                chunks: data.chunks(CHUNK_SIZE).map(|c| c.to_owned()).collect(),
            }
        })
        .collect()
});

macro_rules! create_runner {
    ($settings:expr) => {
        move |b, i: &Vec<Vec<u8>>| {
            b.iter(|| {
                let mut rewriter = lol_html::HtmlRewriter::new($settings, |c: &[u8]| {
                    std::hint::black_box(c);
                });

                for chunk in i {
                    rewriter.write(chunk).unwrap();
                }

                rewriter.end().unwrap();
            })
        }
    };
}

macro_rules! noop_handler {
    () => {
        |arg| {
            std::hint::black_box(arg);
            Ok(())
        }
    };
}

macro_rules! define_group {
    ($group_name:expr, [ $(($name:expr, $settings:expr)),+ ]) => {
        use criterion::*;

        pub fn group(c: &mut Criterion) {
            let mut g = c.benchmark_group($group_name);

            for input in crate::INPUTS.iter() {
                g.throughput(Throughput::Bytes(input.length as u64));

                $(
                    g.bench_with_input(
                        BenchmarkId::new($name, &input.name),
                        &input.chunks,
                        create_runner!($settings),
                    );
                )+
            }

            g.finish();
        }
    };
}

mod cases;

criterion_group!(
    benches,
    cases::parsing::group,
    cases::rewriting::group,
    cases::selector_matching::group
);

criterion_main!(benches);
