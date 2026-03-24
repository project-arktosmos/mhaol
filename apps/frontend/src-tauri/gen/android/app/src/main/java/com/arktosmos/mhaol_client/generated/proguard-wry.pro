# THIS FILE IS AUTO-GENERATED. DO NOT MODIFY!!

# Copyright 2020-2023 Tauri Programme within The Commons Conservancy
# SPDX-License-Identifier: Apache-2.0
# SPDX-License-Identifier: MIT

-keep class com.arktosmos.mhaol_client.* {
  native <methods>;
}

-keep class com.arktosmos.mhaol_client.WryActivity {
  public <init>(...);

  void setWebView(com.arktosmos.mhaol_client.RustWebView);
  java.lang.Class getAppClass(...);
  java.lang.String getVersion();
}

-keep class com.arktosmos.mhaol_client.Ipc {
  public <init>(...);

  @android.webkit.JavascriptInterface public <methods>;
}

-keep class com.arktosmos.mhaol_client.RustWebView {
  public <init>(...);

  void loadUrlMainThread(...);
  void loadHTMLMainThread(...);
  void evalScript(...);
}

-keep class com.arktosmos.mhaol_client.RustWebChromeClient,com.arktosmos.mhaol_client.RustWebViewClient {
  public <init>(...);
}
