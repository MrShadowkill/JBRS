package dev.jbrs.mobile;

import org.libsdl.app.SDLActivity;
import android.view.KeyEvent;
import android.Manifest;
import android.content.pm.PackageManager;
import android.os.Build;
import android.os.Bundle;
import android.os.Environment;
import java.io.File;

/** SDL owns the surface, lifecycle and multi-touch event delivery to native Rust. */
public final class MainActivity extends SDLActivity {
    @Override protected void onCreate(Bundle state) {
        super.onCreate(state);
        prepareShaderPackDirectory();
    }
    private void prepareShaderPackDirectory() {
        if (Build.VERSION.SDK_INT >= 23
                && (checkSelfPermission(Manifest.permission.READ_EXTERNAL_STORAGE) != PackageManager.PERMISSION_GRANTED
                || checkSelfPermission(Manifest.permission.WRITE_EXTERNAL_STORAGE) != PackageManager.PERMISSION_GRANTED)) {
            requestPermissions(new String[] { Manifest.permission.READ_EXTERNAL_STORAGE,
                    Manifest.permission.WRITE_EXTERNAL_STORAGE }, 40);
            return;
        }
        new File(Environment.getExternalStorageDirectory(), "JBRS/shaders/packs").mkdirs();
    }
    @Override public void onRequestPermissionsResult(int requestCode, String[] permissions, int[] results) {
        super.onRequestPermissionsResult(requestCode, permissions, results);
        if (requestCode == 40 && results.length > 0 && results[0] == PackageManager.PERMISSION_GRANTED) {
            new File(Environment.getExternalStorageDirectory(), "JBRS/shaders/packs").mkdirs();
        }
    }
    @Override public void onBackPressed() {
        if (mBrokenLibraries) { super.onBackPressed(); return; }
        // Android 10 gesture Back reaches the activity instead of the Surface key listener.
        onNativeKeyDown(KeyEvent.KEYCODE_BACK);
        onNativeKeyUp(KeyEvent.KEYCODE_BACK);
    }
    @Override protected String[] getLibraries() {
        return new String[] { "SDL2", "jbrs" };
    }
}
