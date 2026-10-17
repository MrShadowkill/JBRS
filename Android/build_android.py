#!/usr/bin/env python3
"""Build a signed ARM64 Android 10+ preview APK from the existing Cargo workspace.
Requires Python 3.12+, stable Rust, CMake, Ninja, Java 17+ and Android SDK/NDK.
No Gradle or third-party Python packages are needed.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tarfile
import urllib.request
import zipfile

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent


def run(args, **kwargs):
    print('+', ' '.join(map(str, args)), flush=True)
    subprocess.run(list(map(str, args)), check=True, **kwargs)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--sdk', default=os.environ.get('ANDROID_SDK_ROOT', os.environ.get('ANDROID_HOME')))
    parser.add_argument('--ndk', default=os.environ.get('ANDROID_NDK_HOME'))
    parser.add_argument('--build-tools', help='SDK build-tools directory (default: latest installed)')
    parser.add_argument('--android-jar', help='Compile SDK 34 android.jar override')
    parser.add_argument('--sdl-source', help='Existing official SDL2 2.30.11 source directory')
    parser.add_argument('--build-dir', default=str(HERE / 'build'))
    parser.add_argument('--keystore', help='Existing signing keystore; otherwise reuse a local preview key')
    parser.add_argument('--key-alias', default='androiddebugkey')
    args = parser.parse_args()
    if not args.sdk or not args.ndk:
        parser.error('Set ANDROID_SDK_ROOT and ANDROID_NDK_HOME, or pass --sdk and --ndk.')
    sdk, ndk, build = Path(args.sdk).resolve(), Path(args.ndk).resolve(), Path(args.build_dir).resolve()
    build.mkdir(parents=True, exist_ok=True)
    if args.build_tools:
        bt = Path(args.build_tools)
    else:
        bt = sorted((sdk / 'build-tools').iterdir(), key=lambda p: [int(n) for n in p.name.split('.')])[-1]
    android_jar = Path(args.android_jar) if args.android_jar else sdk / 'platforms/android-34/android.jar'
    if not android_jar.is_file():
        raise RuntimeError('Install Android SDK platform 34 (compile SDK); min/target remain Android 10/API 29.')
    sources = json.loads((HERE / 'SOURCES.json').read_text())
    sdl = Path(args.sdl_source).resolve() if args.sdl_source else build / ('SDL2-' + sources['sdl_version'])
    if not sdl.is_dir():
        archive = build / 'SDL2-source.tar.gz'
        if not archive.exists():
            urllib.request.urlretrieve(sources['sdl_url'], archive)
        if hashlib.sha256(archive.read_bytes()).hexdigest() != sources['sdl_sha256']:
            raise RuntimeError('SDL2 archive checksum mismatch')
        with tarfile.open(archive) as tar:
            tar.extractall(build, filter='data')
    for tool in ['cargo', 'rustup', 'cmake', 'ninja', 'java', 'keytool']:
        if not shutil.which(tool):
            raise RuntimeError('Required tool not in PATH: ' + tool)
    host = {'linux': 'linux-x86_64', 'darwin': 'darwin-x86_64', 'win32': 'windows-x86_64'}.get(sys.platform)
    if not host:
        raise RuntimeError('Unsupported NDK build host')
    compiler = ndk / 'toolchains/llvm/prebuilt' / host / 'bin/aarch64-linux-android29-clang'
    if sys.platform == 'win32':
        compiler = compiler.with_suffix('.cmd')
    sdl_build = build / 'sdl-arm64'
    run(['cmake', '-S', sdl, '-B', sdl_build, '-G', 'Ninja',
         '-DCMAKE_TOOLCHAIN_FILE=' + str(ndk / 'build/cmake/android.toolchain.cmake'),
         '-DANDROID_ABI=arm64-v8a', '-DANDROID_PLATFORM=android-29', '-DANDROID_STL=c++_static',
         '-DCMAKE_BUILD_TYPE=Release', '-DSDL_SHARED=ON', '-DSDL_STATIC=OFF', '-DSDL_TEST=OFF',
         '-DSDL_HIDAPI=ON', '-DCMAKE_SHARED_LINKER_FLAGS=-Wl,-z,max-page-size=16384'])
    run(['cmake', '--build', sdl_build, '--parallel', '4'])
    run(['rustup', 'target', 'add', 'aarch64-linux-android'])
    env = os.environ.copy()
    env['CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER'] = str(compiler)
    env['CARGO_TARGET_AARCH64_LINUX_ANDROID_RUSTFLAGS'] = '-L native=' + str(sdl_build) + ' -C link-arg=-Wl,-z,max-page-size=16384'
    # Quoted paths in RUSTFLAGS are not portable. A build path without spaces is required.
    if ' ' in str(sdl_build):
        raise RuntimeError('Use a --build-dir path without spaces for native linker search flags.')
    target_dir = build / 'cargo'
    run(['cargo', 'build', '--locked', '--release', '-p', 'jbrs', '--lib',
         '--target', 'aarch64-linux-android', '--target-dir', target_dir], cwd=ROOT, env=env)
    classes, dex = build / 'classes', build / 'dex'
    if classes.exists():
        shutil.rmtree(classes)
    if dex.exists():
        shutil.rmtree(dex)
    classes.mkdir(); dex.mkdir()
    java_files = sorted((sdl / 'android-project/app/src/main/java').rglob('*.java')) + sorted((HERE / 'java').rglob('*.java'))
    run(['java', '-m', 'jdk.compiler/com.sun.tools.javac.Main', '-encoding', 'UTF-8', '-source', '8', '-target', '8', '-g',
         '-classpath', android_jar, '-d', classes, *java_files])
    jar = build / 'classes.jar'
    with zipfile.ZipFile(jar, 'w', zipfile.ZIP_DEFLATED) as z:
        for path in sorted(classes.rglob('*.class')):
            z.write(path, path.relative_to(classes).as_posix())
    run(['java', '-cp', bt / 'lib/d8.jar', 'com.android.tools.r8.D8', '--min-api', '29', '--lib', android_jar, '--output', dex, jar])
    base, unsigned, aligned = build / 'base.apk', build / 'unsigned.apk', build / 'aligned.apk'
    for path in [base, unsigned, aligned]:
        path.unlink(missing_ok=True)
    suffix = '.exe' if sys.platform == 'win32' else ''
    run([bt / ('aapt' + suffix), 'package', '-f', '-M', HERE / 'AndroidManifest.xml', '-S', HERE / 'res', '-I', android_jar, '-F', base])
    shutil.copy2(base, unsigned)
    with zipfile.ZipFile(unsigned, 'a', zipfile.ZIP_DEFLATED) as z:
        for path in sorted(dex.glob('*.dex')):
            z.write(path, path.name)
        z.write(target_dir / 'aarch64-linux-android/release/libjbrs.so', 'lib/arm64-v8a/libjbrs.so')
        z.write(sdl_build / 'libSDL2.so', 'lib/arm64-v8a/libSDL2.so')
        shader_root = ROOT / 'shaders'
        if shader_root.is_dir():
            for path in sorted(shader_root.rglob('*')):
                if path.is_file():
                    z.write(path, 'assets/shaders/' + path.relative_to(shader_root).as_posix())
        for path, name in [(ROOT / 'LICENSE', 'LICENSE'), (ROOT / 'THIRD_PARTY_NOTICES.md', 'THIRD_PARTY_NOTICES.md'),
                           (ROOT / 'vehicles/assets/ATTRIBUTION.md', 'VEHICLE_ATTRIBUTION.md'),
                           (ROOT / 'renderer_gles2/assets/FONT-LICENSE.txt', 'FONT-LICENSE.txt'),
                           (sdl / 'LICENSE.txt', 'SDL2-LICENSE.txt')]:
            z.write(path, 'assets/licenses/' + name)
    run([bt / ('zipalign' + suffix), '-f', '4', unsigned, aligned])
    # The local preview key is intentionally outside the repository by default.
    keystore = Path(args.keystore).resolve() if args.keystore else Path.home() / '.cache/jbrs/android-preview.keystore'
    if not keystore.exists():
        if args.keystore:
            raise RuntimeError('Requested signing keystore does not exist')
        keystore.parent.mkdir(parents=True, exist_ok=True)
        run(['keytool', '-genkeypair', '-keystore', keystore, '-alias', args.key_alias, '-storepass', 'android',
             '-keypass', 'android', '-keyalg', 'RSA', '-keysize', '2048', '-validity', '3650', '-dname', 'CN=JBRS Local Preview'])
    env = os.environ.copy()
    # For a private production key provide password environment variables, never commit the key.
    env.setdefault('JBRS_STORE_PASSWORD', 'android'); env.setdefault('JBRS_KEY_PASSWORD', 'android')
    apk = HERE / 'dist/JBRS-Android10-arm64.apk'
    apk.parent.mkdir(exist_ok=True)
    Path(str(apk) + '.idsig').unlink(missing_ok=True)
    signer = ['java', '-jar', bt / 'lib/apksigner.jar']
    run([*signer, 'sign', '--ks', keystore, '--ks-key-alias', args.key_alias, '--ks-pass', 'env:JBRS_STORE_PASSWORD',
         '--key-pass', 'env:JBRS_KEY_PASSWORD', '--v4-signing-enabled', 'false', '--out', apk, aligned], env=env)
    run([*signer, 'verify', '--verbose', '--print-certs', apk])
    digest = hashlib.sha256(apk.read_bytes()).hexdigest()
    (apk.parent / 'SHA256SUMS').write_text(digest + '  ' + apk.name + '\n')
    print('APK ready:', apk)


if __name__ == '__main__':
    try:
        main()
    except (OSError, RuntimeError, subprocess.CalledProcessError) as exc:
        raise SystemExit('Android build failed: ' + str(exc))
