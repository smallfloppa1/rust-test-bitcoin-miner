use sha2::{Digest, Sha256};
use hex;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::thread;

fn double_sha256(input: &str) -> [u8; 32] {
    let mut hasher1 = Sha256::new();
    hasher1.update(input.as_bytes());
    let first_hash = hasher1.finalize();

    let mut hasher2 = Sha256::new();
    hasher2.update(first_hash);

    hasher2.finalize().into()
}

fn miner_thread(
    thread_id: u32,
    start_nonce: u32,
    step: u32,
    block_header: String,
    difficulty_target: String,
    found_solution: Arc<AtomicBool>,
) {
    let mut nonce = start_nonce;

    println!("Thread {} started, checking nonces starting at {} (step {})", thread_id, start_nonce, step);

    loop {
        if found_solution.load(Ordering::Relaxed) {
            break;
        }

        let data_to_hash = format!("{}{}", block_header, nonce);
        let hash_bytes = double_sha256(&data_to_hash);
        let hash_hex = hex::encode(hash_bytes);

        if hash_hex.starts_with(&difficulty_target) {
            found_solution.store(true, Ordering::Relaxed);

            println!("\n✅ Thread {} found the block!", thread_id);
            println!("Nonce:   {}", nonce);
            println!("Hash:    {}", hash_hex);
            break;
        }

        if nonce.checked_sub(start_nonce).map_or(false, |diff| diff % (100000 * step) == 0) {
            if !found_solution.load(Ordering::Relaxed) {
                println!("Thread {} has checked at least {} hashes...", thread_id, nonce.saturating_sub(start_nonce));
            }
        }

        nonce = nonce.wrapping_add(step);
    }
}

fn main() {
    let num_threads = thread::available_parallelism().map(|n| n.get()).unwrap_or(1) as u32;

    let found_solution = Arc::new(AtomicBool::new(false));

    let block_header = String::from("Version: 1, PreviousHash: 0000abc..., MerkleRoot: xyz..., Timestamp: 123456789, Bits: 486604799, Nonce: ");
    let difficulty_target = String::from("0000000");

    println!("Starting multi-threaded Rust miner ({} threads)...", num_threads);
    println!("Target: Find a hash starting with '{}'", difficulty_target);
    println!("----------------------------------------");

    let mut handles = Vec::new();

    let step = num_threads;

    for i in 0..num_threads {
        let start_nonce = i;

        let header_clone = block_header.clone();
        let target_clone = difficulty_target.clone();
        let found_clone = Arc::clone(&found_solution);

        let handle = thread::spawn(move || {
            miner_thread(
                i + 1,
                start_nonce,
                step,
                header_clone,
                target_clone,
                found_clone,
            );
        });
        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.join();
    }

    println!("\nMining session concluded.");
}
