import 'dart:async';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_inappwebview/flutter_inappwebview.dart';
import 'package:http/http.dart' as http;
import 'linux_manager.dart';
import 'terminal_screen.dart';
import 'log_viewer_sheet.dart';

// Start a local server to serve WebAssembly and local assets properly with correct MIME types
final InAppLocalhostServer localServer = InAppLocalhostServer(port: 8080, documentRoot: 'assets/www');

void main() async {
  WidgetsFlutterBinding.ensureInitialized();

  // Set system UI style for a modern premium feel
  SystemChrome.setSystemUIOverlayStyle(const SystemUiOverlayStyle(
    statusBarColor: Colors.transparent,
    statusBarIconBrightness: Brightness.light,
    statusBarBrightness: Brightness.dark,
    systemNavigationBarColor: Color(0xFF181818), // Matches webapp background
    systemNavigationBarIconBrightness: Brightness.light,
  ));

  // Initialize background Linux environment and Codedroid API
  await LinuxManager.initialize();

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
      theme: ThemeData(
        useMaterial3: true,
        brightness: Brightness.dark,
        scaffoldBackgroundColor: const Color(0xFF181818), // Match webapp background exactly (#181818)
        colorScheme: const ColorScheme.dark(
          primary: Color(0xFF228DF2), // Accent blue
          secondary: Color(0xFF15AC91), // Accent green
          surface: Color(0xFF1D1D1D),
        ),
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

  // --- API Status ---
  bool _apiRunning = false;
  Timer? _apiCheckTimer;

  @override
  void initState() {
    super.initState();
    _checkApiStatus();
    _apiCheckTimer = Timer.periodic(const Duration(seconds: 3), (_) => _checkApiStatus());
  }

  @override
  void dispose() {
    _apiCheckTimer?.cancel();
    super.dispose();
  }

  Future<void> _checkApiStatus() async {
    try {
      final response = await http
          .get(Uri.parse('http://localhost:3000/ping'))
          .timeout(const Duration(seconds: 2));
      if (mounted) {
        setState(() => _apiRunning = response.statusCode == 200);
      }
    } catch (_) {
      if (mounted) setState(() => _apiRunning = false);
    }
  }

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
            Positioned(
              bottom: 12,
              left: 88,
              child: GestureDetector(
                onTap: () => LogViewerSheet.show(context, _apiRunning),
                child: AnimatedContainer(
                  duration: const Duration(milliseconds: 400),
                  padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 5),
                  decoration: BoxDecoration(
                    color: _apiRunning
                        ? Colors.green.withOpacity(0.85)
                        : Colors.red.withOpacity(0.85),
                    borderRadius: BorderRadius.circular(20),
                    boxShadow: const [BoxShadow(color: Colors.black38, blurRadius: 6, offset: Offset(0, 2))],
                  ),
                  child: Row(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      Icon(
                        _apiRunning ? Icons.check_circle : Icons.cancel,
                        color: Colors.white,
                        size: 14,
                      ),
                      const SizedBox(width: 5),
                      Text(
                        _apiRunning ? 'API: Running' : 'API: Not Running',
                        style: const TextStyle(
                          color: Colors.white,
                          fontSize: 12,
                          fontWeight: FontWeight.w600,
                        ),
                      ),
                    ],
                  ),
                ),
              ),
            ),
            if (isLoading)
              Center(
                child: CircularProgressIndicator(
                  color: Theme.of(context).colorScheme.primary,
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
                          backgroundColor: Theme.of(context).colorScheme.primary,
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
      floatingActionButton: FloatingActionButton(
        onPressed: () {
          Navigator.push(
            context,
            MaterialPageRoute(
              builder: (context) => const InteractiveTerminalScreen(),
            ),
          );
        },
        backgroundColor: Theme.of(context).colorScheme.primary,
        child: const Icon(Icons.terminal, color: Colors.black),
      ),
      floatingActionButtonLocation: FloatingActionButtonLocation.startFloat,
    );
  }
}
