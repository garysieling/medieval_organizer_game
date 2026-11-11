// The page's sound layer: each frame's sfx_v cues become little WebAudio
// voices. The voice table mirrors record.rs's `synth_track` - the same
// kinds, the same partials, so a bell in a video sounds like a bell on
// the page. Audio needs a user gesture to start; the first click or key
// calls resume().

const TAU = Math.PI * 2;

export class SoundBoard {
  constructor() { this.ctx = null; this.master = null; this.noise = null; }

  resume() {
    if (!this.ctx) {
      const AC = window.AudioContext || window.webkitAudioContext;
      if (!AC) return;
      this.ctx = new AC();
      this.master = this.ctx.createGain();
      this.master.gain.value = 0.25;
      this.master.connect(this.ctx.destination);
      const n = this.ctx.sampleRate;
      this.noise = this.ctx.createBuffer(1, n, n);
      const d = this.noise.getChannelData(0);
      for (let i = 0; i < n; i++) d[i] = Math.random() * 2 - 1;
    }
    if (this.ctx.state === 'suspended') this.ctx.resume();
  }

  // One cue: kind names the voice, pan -1..1 by where it fell on screen,
  // gain the cue's intensity. Mirrors voice()/voice_len in record.rs.
  cue(kind, pan, gain) {
    if (!this.ctx || this.ctx.state !== 'running') return;
    const now = this.ctx.currentTime;
    const out = this.ctx.createStereoPanner();
    out.pan.value = Math.max(-1, Math.min(1, pan));
    const g = this.ctx.createGain();
    g.gain.value = Math.min(1, gain) * 0.8;
    out.connect(g); g.connect(this.master);

    const tone = (freq, amp, len, at = 0) => {
      const o = this.ctx.createOscillator();
      const e = this.ctx.createGain();
      o.frequency.value = freq;
      e.gain.setValueAtTime(0, now + at);
      e.gain.linearRampToValueAtTime(amp, now + at + 0.01);
      e.gain.exponentialRampToValueAtTime(0.001, now + at + len);
      o.connect(e); e.connect(out);
      o.start(now + at); o.stop(now + at + len + 0.05);
    };
    const hiss = (len, amp) => {
      const s = this.ctx.createBufferSource();
      s.buffer = this.noise;
      const f = this.ctx.createBiquadFilter();
      f.type = 'lowpass'; f.frequency.value = 800;
      const e = this.ctx.createGain();
      e.gain.setValueAtTime(amp, now);
      e.gain.exponentialRampToValueAtTime(0.001, now + len);
      s.connect(f); f.connect(e); e.connect(out);
      s.start(now); s.stop(now + len);
    };

    switch (kind) {
      case 'rain':     hiss(0.8, 0.4); break;
      case 'anvil':    tone(815, 0.5, 0.4); tone(2130, 0.25, 0.4); break;
      case 'festival': tone(392, 0.3, 1.4); tone(494, 0.3, 1.4); tone(587, 0.3, 1.4); break;
      case 'bell':     tone(220, 0.6, 2.0); tone(523, 0.35, 2.0); tone(891, 0.25, 2.0); break;
      case 'wonder':   tone(1175, 0.3, 1.5); tone(1175, 0.2, 1.5, 0.15); break;
      case 'snowballs': tone(660, 0.3, 0.9); tone(880, 0.3, 0.9, 0.06); break;
      case 'splash':   hiss(0.3, 0.4); break;
      default:         tone(520, 0.25, 0.25); break;
    }
  }

  // Drain the cues a fresh scene raised - each [[kind, pan, gain], ...]
  drain(cues) {
    for (const [kind, pan, gain] of cues) this.cue(kind, pan, gain);
  }
}
