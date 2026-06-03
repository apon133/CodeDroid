import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_inappwebview/flutter_inappwebview.dart';

// Start a local server to serve WebAssembly and local assets properly with correct MIME types
final InAppLocalhostServer localServer = InAppLocalhostServer(port: 8080, documentRoot: 'assets/www');

void main() async {
  WidgetsFlutterBinding.ensureInitialized();

  // Set system UI style for a modern premium feel
  SystemChrome.setSystemUIOverlayStyle(const SystemUiOverlayStyle(
    statusBarColor: Colors.transparent,
    statusBarIconBrightness: Brightness.light,
    systemNavigationBarColor: Colors.black,
    systemNavigationBarIconBrightness: Brightness.light,
  ));

  try {
    await localServer.start();
  } catch (e) {
    debugPrint("Local server failed to start: $e");
  }

  runApp(const CodeDroidApp());
}

class CodeDroidApp extends StatelessWidget {
  const CodeDroidApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'CodeDroid',
      debugShowCheckedModeBanner: false,
      theme: ThemeData.dark().copyWith(
        scaffoldBackgroundColor: const Color(0xFF1E1E2E), // Premium dark theme matching CodeDroid
      ),
      home: const WebViewContainer(),
    );
  }
}

class WebViewContainer extends StatefulWidget {
  const WebViewContainer({super.key});

  @override
  State<WebViewContainer> createState() => _WebViewContainerState();
}

class _WebViewContainerState extends State<WebViewContainer> {
  InAppWebViewController? webViewController;
  bool isLoading = true;
  String? loadError;

  @override
  Widget build(BuildContext context) {
    // Determine the entry point URL for the local server
    final String entryUrl = "http://localhost:8080/";

    return Scaffold(
      body: SafeArea(
        child: Stack(
          children: [
            InAppWebView(
              initialUrlRequest: URLRequest(
                url: WebUri(entryUrl),
              ),
              initialSettings: InAppWebViewSettings(
                // Performance and features
                useShouldOverrideUrlLoading: true,
                mediaPlaybackRequiresUserGesture: false,
                
                // Cache, Cookies, and Local Storage
                javaScriptEnabled: true,
                domStorageEnabled: true,
                databaseEnabled: true,
                cacheEnabled: true,
                clearCache: false, // Ensure caching stays active
                
                // File pickers and local access permissions
                allowFileAccess: true,
                allowContentAccess: true,
                allowFileAccessFromFileURLs: true,
                allowUniversalAccessFromFileURLs: true,
                
                // Visual properties
                verticalScrollBarEnabled: false,
                horizontalScrollBarEnabled: false,
                supportZoom: false,
              ),
              onWebViewCreated: (controller) {
                webViewController = controller;
              },
              onLoadStart: (controller, url) {
                setState(() {
                  isLoading = true;
                  loadError = null;
                });
              },
              onLoadStop: (controller, url) {
                setState(() {
                  isLoading = false;
                });
              },
              onReceivedError: (controller, request, error) {
                // Ignore minor errors or specific resource load errors
                if (request.isForMainFrame ?? false) {
                  setState(() {
                    isLoading = false;
                    loadError = error.description;
                  });
                }
              },
              // Handle file picker requests automatically (camera, file system, etc.)
              onPermissionRequest: (controller, permissionRequest) async {
                return PermissionResponse(
                  resources: permissionRequest.resources,
                  action: PermissionResponseAction.GRANT,
                );
              },
            ),
            if (isLoading)
              const Center(
                child: CircularProgressIndicator(
                  color: Color(0xFF89B4FA), // Cute pastel blue accent
                ),
              ),
            if (loadError != null)
              Center(
                child: Padding(
                  padding: const EdgeInsets.all(24.0),
                  child: Column(
                    mainAxisAlignment: MainAxisAlignment.center,
                    children: [
                      const Icon(Icons.error_outline, color: Colors.redAccent, size: 64),
                      const SizedBox(height: 16),
                      Text(
                        'Failed to load application',
                        style: Theme.of(context).textTheme.titleLarge?.copyWith(color: Colors.white),
                      ),
                      const SizedBox(height: 8),
                      Text(
                        loadError!,
                        style: Theme.of(context).textTheme.bodyMedium?.copyWith(color: Colors.grey),
                        textAlign: TextAlign.center,
                      ),
                      const SizedBox(height: 24),
                      ElevatedButton.icon(
                        onPressed: () {
                          webViewController?.reload();
                        },
                        icon: const Icon(Icons.refresh),
                        label: const Text('Retry'),
                        style: ElevatedButton.styleFrom(
                          backgroundColor: const Color(0xFF89B4FA),
                          foregroundColor: Colors.black,
                        ),
                      ),
                    ],
                  ),
                ),
              ),
          ],
        ),
      ),
    );
  }
}
