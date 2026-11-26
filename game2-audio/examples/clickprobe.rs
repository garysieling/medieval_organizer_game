use game2_audio::{analysis, sampler};

fn main() {
    for path in std::env::args().skip(1) {
        let bytes = std::fs::read(&path).unwrap();
        let s = sampler::decode(&bytes).unwrap();
        let stereo: Vec<f32> = s.pcm.iter().flat_map(|&v| [v, v]).collect();
        let clicks = analysis::clicks(&stereo, s.sr);
        println!("{}: {} clicks ({} sr, {:.1}s)", path, clicks.len(), s.sr, s.pcm.len() as f32 / s.sr as f32);
    }
}
