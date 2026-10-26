package dev.jbrs.uglglith;

import org.libsdl.app.SDLActivity;
import android.os.Bundle;
import android.view.KeyEvent;
import java.io.File;

public final class MainActivity extends SDLActivity {
    @Override protected void onCreate(Bundle state) {
        super.onCreate(state);
        new File(getFilesDir(), "shaders/packs").mkdirs();
    }
    @Override public void onBackPressed() {
        if (mBrokenLibraries) { super.onBackPressed(); return; }
        onNativeKeyDown(KeyEvent.KEYCODE_BACK);
        onNativeKeyUp(KeyEvent.KEYCODE_BACK);
    }
    @Override protected String[] getLibraries() { return new String[] { "SDL2", "jbrs" }; }
}
