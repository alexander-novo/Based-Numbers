use std::{error::Error, fmt::Write, path::PathBuf, time::Duration};

use based_num::TinyMap;
use clap::Parser;
use csv::Writer;
use indicatif::{ProgressBar, ProgressIterator, ProgressState, ProgressStyle};
use serde::Serialize;

#[derive(Parser)]
/// Calculate basedness for all numbers from 1 to a certain maximum (see MAX_NUM),
/// then output the sequence of based numbers until that maximum.
struct Args {
    #[arg(default_value_t = 100_000_000)]
    /// The maximum number to check basedness of.
    max_num: u64,

    #[arg(short, long)]
    /// Output calculated number info for numbers considered.
    output_csv: Option<PathBuf>,

    #[arg(short, long)]
    /// Histogram of prime factor distribution
    prime_factor_csv: Option<PathBuf>,
}

#[derive(Clone, Copy, Debug, Serialize)]
struct NumProperties {
    number: u64,
    num_factors: u64,
    num_prime_factors: u64,
    basedness: u64,
}

/// A multiset of prime factors. Represented as a map of Prime -> Power.
/// Backing storage of `TinyMap` ensures that as long as there are 3 or fewer
/// prime factors for a number (which is true for ~62% of numbers),
/// this will not need to allocate. To reduce allocations more, increase
/// the size of the array part of the `TinyMap` - doing this will increase the
/// amount of memory used for numbers with fewer than that many factors.
/// For a backing storage array size of 3, there will not be any need for allocation
/// for ~62% of numbers, but the average amount of memory used will be increased by
/// ~22%
type FactorMultiset = TinyMap<usize, u32, 3>;

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let n = (args.max_num + 1) as usize;

    let mut num_properties = vec![None; n];
    let mut prime_factors = vec![FactorMultiset::new(); n];
    let mut primes = Vec::new();
    let mut based = Vec::new();

    let mut num_prime_factors_histogram = [0; 10];

    println!(
        "Size of FactorMultiset: {}",
        std::mem::size_of::<FactorMultiset>()
    );

    num_properties[1] = Some(NumProperties {
        number: 1,
        num_factors: 1,
        num_prime_factors: 0,
        basedness: 0,
    });

    for i in progress_bar(2..n) {
        // Find a prime factor of i - it must necessarily be one of the primes we have already found,
        // or i is itself a prime
        let p = primes
            .iter()
            .copied()
            // If i is non-prime, then one of its factors must be no larger than sqrt(i)
            .take_while(|p| p * p <= i)
            .find(|p| i % p == 0);

        let num_factors =
        // If we found some small (< i) prime factor p
        if let Some(p) = p {
            // All factors of i / p are also factors of i
            prime_factors[i] = prime_factors[i / p].clone();

            // The power of p in the prime factor representation of i is
            // 1 + the power of p in the prime factor representation of i / p
            prime_factors[i]
                .entry(p)
                .and_modify(|k| *k += 1)
                .or_insert(1);

            // Definition of d(n) the divisor function
            prime_factors[i].values().copied().map(|k| u64::from(k + 1)).product()
        // Otherwise, i must be a prime
        } else {
            prime_factors[i].insert(i, 1);
            primes.push(i);

            // All prime numbers have 2 factors: 1 and itself
            2
        };
        prime_factors[i].shrink_to_fit();

        let num_prime_factors = prime_factors[i].len() as u64;

        num_prime_factors_histogram[num_prime_factors as usize - 1] += 1;

        let basedness = num_prime_factors * num_properties[i - 1].unwrap().num_factors;
        num_properties[i] = Some(NumProperties {
            number: i as u64,
            num_factors,
            num_prime_factors,
            basedness,
        });

        // A based number is one which is square-free and is more based than all smaller based numbers
        if basedness > based.last().copied().map_or(0, |(_, basedness)| basedness) {
            based.push((i, basedness));
        }
    }

    let num_prime_factors_histogram = num_prime_factors_histogram
        .iter()
        .copied()
        .take_while(|n| *n > 0)
        .enumerate()
        .map(|(i, n)| (i + 1, n))
        .collect::<Vec<_>>();

    println!("Based numbers:");
    println!("{based:?}");
    println!("Prime factor histogram:");
    println!("{num_prime_factors_histogram:?}",);

    if let Some(path) = args.output_csv {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let mut wtr = Writer::from_path(path)?;

        for prop in num_properties.iter().flatten() {
            wtr.serialize(prop)?;
        }
    }

    if let Some(path) = args.prime_factor_csv {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let mut wtr = Writer::from_path(path)?;

        for bucket in num_prime_factors_histogram.iter() {
            wtr.serialize(bucket)?;
        }
    }

    Ok(())
}

fn progress_bar<T>(iter: impl ExactSizeIterator<Item = T>) -> impl Iterator<Item = T> {
    let pb = ProgressBar::new(iter.len() as u64);

    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} ({eta})",
        )
        .unwrap()
        .with_key("eta", |state: &ProgressState, w: &mut dyn Write| {
            write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap()
        })
        .progress_chars("#>-")
        .tick_chars("◐◐◓◓◑◑◒◒◐◐"),
    );

    pb.enable_steady_tick(Duration::from_millis(125));

    iter.progress_with(pb)
}
