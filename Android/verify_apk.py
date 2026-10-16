#!/usr/bin/env python3
"""Validate the produced APK's API 29 signature, manifest, DEX and ARM64 linkage.
This is packaging verification, not a device runtime test.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess
import tempfile
import zipfile


def output(args):
    return subprocess.check_output(list(map(str, args)), stderr=subprocess.STDOUT, text=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--apk', type=Path, default=Path(__file__).parent / 'dist/JBRS-Android10-arm64.apk')
    parser.add_argument('--build-tools', type=Path, required=True)
    parser.add_argument('--llvm-bin', type=Path, required=True)
    parser.add_argument('--output', type=Path)
    args = parser.parse_args()
    apk, bt, llvm = args.apk, args.build_tools, args.llvm_bin
    signing = output(['java', '-jar', bt / 'lib/apksigner.jar', 'verify', '--verbose',
                      '--min-sdk-version', '29', '--max-sdk-version', '29', apk])
    badging = output([bt / 'aapt', 'dump', 'badging', apk])
    for required in ["name='dev.jbrs.mobile'", "sdkVersion:'29'", "targetSdkVersion:'29'",
                     "uses-gl-es: '0x20000'", "native-code: 'arm64-v8a'", 'dev.jbrs.mobile.MainActivity']:
        if required not in badging:
            raise RuntimeError('Unexpected APK manifest: missing ' + required)
    report = {'apk': apk.name, 'sha256': hashlib.sha256(apk.read_bytes()).hexdigest(),
              'api': 29, 'abi': 'arm64-v8a', 'signature_verification': signing.strip(), 'libraries': {}}
    allowed = {'libSDL2.so', 'libGLESv2.so', 'libandroid.so', 'liblog.so', 'libdl.so', 'libm.so', 'libc.so', 'libOpenSLES.so', 'libGLESv1_CM.so'}
    with zipfile.ZipFile(apk) as z, tempfile.TemporaryDirectory(prefix='jbrs-apk-') as temp:
        if z.testzip() is not None:
            raise RuntimeError('APK CRC failure')
        if not z.read('classes.dex').startswith(b'dex\n'):
            raise RuntimeError('Missing Android DEX bytecode')
        for license_name in ['LICENSE', 'THIRD_PARTY_NOTICES.md', 'VEHICLE_ATTRIBUTION.md', 'FONT-LICENSE.txt', 'SDL2-LICENSE.txt']:
            z.read('assets/licenses/' + license_name)
        for name in ['libjbrs.so', 'libSDL2.so']:
            data = z.read('lib/arm64-v8a/' + name)
            if data[:6] != b'\x7fELF\x02\x01' or int.from_bytes(data[18:20], 'little') != 183:
                raise RuntimeError('Library is not ELF64 little-endian AArch64: ' + name)
            path = Path(temp) / name
            path.write_bytes(data)
            needed = re.findall(r'NEEDED.*?\[(.*?)\]', output([llvm / 'llvm-readelf', '-d', path]))
            if set(needed) - allowed:
                raise RuntimeError('Unexpected Android native dependencies: ' + repr(needed))
            symbols = output([llvm / 'llvm-nm', '-D', '--defined-only', path])
            required = ['SDL_main'] if name == 'libjbrs.so' else [
                'Java_org_libsdl_app_SDLActivity_nativeSetupJNI',
                'Java_org_libsdl_app_SDLActivity_nativeRunMain',
                'Java_org_libsdl_app_HIDDeviceManager_HIDDeviceRegisterCallback']
            for symbol in required:
                if not re.search(r'\b' + re.escape(symbol) + r'$', symbols, re.M):
                    raise RuntimeError('Missing native entry point: ' + symbol)
            report['libraries'][name] = {'needed': needed, 'entry_points_verified': required}
    report['runtime_on_android_device'] = 'not tested'
    encoded = json.dumps(report, indent=2) + '\n'
    if args.output:
        args.output.write_text(encoded)
    print(encoded)


if __name__ == '__main__':
    main()
