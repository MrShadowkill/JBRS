"""Reconstruct uploaded sine functions offline; Python 3 standard library only."""
import json
import math
from pathlib import Path
import struct

ROOT = Path(__file__).resolve().parent
report = {}
for source in sorted((ROOT / 'source').glob('*.json')):
    doc = json.loads(source.read_text())
    assert doc['schema'] == 'jbrs.sound.v1' and doc['sampleRate'] == 22050
    assert doc['encoding'] == 'sine-frames' and doc['channels'] == 1
    length = doc['length']
    assert 0 < length <= 22050 * 60
    functions = doc['functions']
    assert functions['fftSize'] == 1024 and functions['hop'] == 512
    assert functions['window'] == 'hann-periodic'
    assert len(functions['frames']) == math.ceil(length / 512) + 1
    out, weights = [0.] * length, [0.] * length
    window = [0.5 - 0.5 * math.cos(math.tau * i / 1024) for i in range(1024)]
    for index, frame in enumerate(functions['frames']):
        assert frame['start'] == (index - 1) * 512 and len(frame['terms']) <= 32
        for term in frame['terms']:
            k, amplitude, phase = term
            assert isinstance(k, int) and 1 <= k < 512
            assert math.isfinite(amplitude) and 0 <= amplitude <= 2
            assert math.isfinite(phase) and abs(phase) <= 10
        assert all(math.isfinite(frame[key]) and abs(frame[key]) <= 1 for key in ['dc','nyquist'])
        for i in range(1024):
            p = frame['start'] + i
            if not 0 <= p < length:
                continue
            value = frame['dc'] + frame['nyquist'] * (-1 if i % 2 else 1)
            value += sum(a * math.sin(math.tau * k * i / 1024 + phase) for k, a, phase in frame['terms'])
            out[p] += value * window[i]
            weights[p] += window[i] ** 2
    out = [max(-1., min(1., x / max(w, 1e-8))) for x, w in zip(out, weights)]
    effects = doc['effects']
    assert -24 <= effects['semitones'] <= 12 and 0 <= effects['gain'] <= 1
    rate = 2 ** (effects['semitones'] / 12)
    data = []
    for i in range(math.ceil(length / rate)):
        p = i * rate
        j = int(p)
        f = p - j
        data.append((out[j] * (1-f) + (out[j+1] if j+1 < length else 0.) * f) * effects['gain'])
    # Remove only near-silent leading/trailing padding, preserving the edited timbre.
    peak = max(map(abs, data))
    active = [i for i, v in enumerate(data) if abs(v) > max(peak * 0.008, 1e-7)]
    assert active, source
    data = data[max(0, active[0]-110):min(len(data),active[-1]+111)]
    for i in range(min(110, len(data)//2)):
        data[i] *= i/110
        data[-1-i] *= i/110
    (ROOT / (source.stem + '.pcm')).write_bytes(b''.join(struct.pack('<h', round(max(-1.,min(1.,v))*32767)) for v in data))
    report[source.stem] = {'source_name':doc['name'],'effects':effects,'samples':len(data),'peak':max(map(abs,data)),'rms':math.sqrt(sum(v*v for v in data)/len(data))}
(ROOT / 'manifest.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n')
print(json.dumps(report,ensure_ascii=False,indent=2))
