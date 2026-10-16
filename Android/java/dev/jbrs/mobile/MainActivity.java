package dev.jbrs.mobile;

import org.libsdl.app.SDLActivity;
import android.view.KeyEvent;

/** SDL owns the surface, lifecycle and multi-touch event delivery to native Rust. */
public final class MainActivity extends SDLActivity {
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
