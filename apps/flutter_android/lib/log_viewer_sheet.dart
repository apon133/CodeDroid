import 'dart:convert';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:http/http.dart' as http;
import 'linux_manager.dart';

class LogViewerSheet extends StatefulWidget {
  final bool apiRunning;
  const LogViewerSheet({super.key, required this.apiRunning});

  static void show(BuildContext context, bool apiRunning) {
    showModalBottomSheet(
      context: context,
      isScrollControlled: true,
      backgroundColor: Colors.transparent,
      builder: (context) => LogViewerSheet(apiRunning: apiRunning),
    );
  }

  @override
  State<LogViewerSheet> createState() => _LogViewerSheetState();
}

class _LogViewerSheetState extends State<LogViewerSheet> with SingleTickerProviderStateMixin {
  late TabController _tabController;
  List<String> rustLogs = [];
  bool isLoadingRustLogs = false;
  String? rustLogsError;

  @override
  void initState() {
    super.initState();
    _tabController = TabController(length: 2, vsync: this);
    if (widget.apiRunning) {
      _fetchRustLogs();
    }
  }

  @override
  void dispose() {
    _tabController.dispose();
    super.dispose();
  }

  Future<void> _fetchRustLogs() async {
    setState(() {
      isLoadingRustLogs = true;
      rustLogsError = null;
    });
    try {
      final response = await http.get(Uri.parse('http://localhost:3000/logs'));
      if (response.statusCode == 200) {
        final List<dynamic> decoded = json.decode(response.body);
        setState(() {
          rustLogs = decoded.map((e) => e.toString()).toList();
          isLoadingRustLogs = false;
        });
      } else {
        setState(() {
          rustLogsError = "Failed to load: Status code ${response.statusCode}";
          isLoadingRustLogs = false;
        });
      }
    } catch (e) {
      setState(() {
        rustLogsError = "Error connecting to API: $e";
        isLoadingRustLogs = false;
      });
    }
  }

  void _copyToClipboard(List<String> logs) {
    final text = logs.join('\n');
    Clipboard.setData(ClipboardData(text: text));
    ScaffoldMessenger.of(context).showSnackBar(
      const SnackBar(
        content: Text('Logs copied to clipboard'),
        duration: Duration(seconds: 2),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final localLogs = LinuxManager.processLogs;

    return Container(
      height: MediaQuery.of(context).size.height * 0.85,
      decoration: const BoxDecoration(
        color: Color(0xFF1E1E1E),
        borderRadius: BorderRadius.vertical(top: Radius.circular(24)),
        border: Border(
          top: BorderSide(color: Color(0xFF333333), width: 1.5),
        ),
      ),
      child: Column(
        children: [
          // Handle/Drag bar
          const SizedBox(height: 12),
          Container(
            width: 48,
            height: 4,
            decoration: BoxDecoration(
              color: Colors.grey[600],
              borderRadius: BorderRadius.circular(2),
            ),
          ),
          const SizedBox(height: 12),
          // Title Bar
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16.0),
            child: Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Row(
                  children: [
                    const Icon(Icons.receipt_long, color: Color(0xFF228DF2)),
                    const SizedBox(width: 8),
                    Text(
                      'System & LSP Logs',
                      style: Theme.of(context).textTheme.titleLarge?.copyWith(
                            color: Colors.white,
                            fontWeight: FontWeight.bold,
                          ),
                    ),
                  ],
                ),
                IconButton(
                  icon: const Icon(Icons.close, color: Colors.grey),
                  onPressed: () => Navigator.pop(context),
                ),
              ],
            ),
          ),
          // Tab bar
          TabBar(
            controller: _tabController,
            indicatorColor: const Color(0xFF228DF2),
            labelColor: const Color(0xFF228DF2),
            unselectedLabelColor: Colors.grey,
            tabs: const [
              Tab(text: 'Process Console'),
              Tab(text: 'Rust API Logs'),
            ],
          ),
          // Content
          Expanded(
            child: TabBarView(
              controller: _tabController,
              children: [
                // Local Process logs
                _buildLogsView(
                  logs: localLogs,
                  onRefresh: () => setState(() {}),
                  emptyMessage: 'No process logs captured yet.',
                ),
                // Rust API Logs
                widget.apiRunning
                    ? isLoadingRustLogs
                        ? const Center(
                            child: CircularProgressIndicator(
                              color: Color(0xFF228DF2),
                            ),
                          )
                        : rustLogsError != null
                            ? Center(
                                child: Padding(
                                  padding: const EdgeInsets.all(24.0),
                                  child: Text(
                                    rustLogsError!,
                                    style: const TextStyle(color: Colors.redAccent),
                                    textAlign: TextAlign.center,
                                  ),
                                ),
                              )
                            : _buildLogsView(
                                logs: rustLogs,
                                onRefresh: _fetchRustLogs,
                                emptyMessage: 'No Rust API internal logs yet.',
                              )
                    : const Center(
                        child: Text(
                          'Rust API is not running.\nStart the API to view internal logs.',
                          style: TextStyle(color: Colors.grey),
                          textAlign: TextAlign.center,
                        ),
                      ),
              ],
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildLogsView({
    required List<String> logs,
    required VoidCallback onRefresh,
    required String emptyMessage,
  }) {
    if (logs.isEmpty) {
      return Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            const Icon(Icons.info_outline, size: 48, color: Colors.grey),
            const SizedBox(height: 16),
            Text(emptyMessage, style: const TextStyle(color: Colors.grey)),
            const SizedBox(height: 16),
            ElevatedButton.icon(
              onPressed: onRefresh,
              icon: const Icon(Icons.refresh),
              label: const Text('Refresh'),
              style: ElevatedButton.styleFrom(
                backgroundColor: const Color(0xFF2E2E2E),
                foregroundColor: Colors.white,
              ),
            )
          ],
        ),
      );
    }

    return Column(
      children: [
        // Action Bar for Logs
        Container(
          padding: const EdgeInsets.symmetric(horizontal: 16.0, vertical: 8.0),
          color: const Color(0xFF151515),
          child: Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              Text(
                '${logs.length} Lines captured',
                style: const TextStyle(color: Colors.grey, fontSize: 13),
              ),
              Row(
                children: [
                  IconButton(
                    icon: const Icon(Icons.copy, size: 18, color: Colors.grey),
                    onPressed: () => _copyToClipboard(logs),
                    tooltip: 'Copy all to clipboard',
                  ),
                  IconButton(
                    icon: const Icon(Icons.refresh, size: 18, color: Colors.grey),
                    onPressed: onRefresh,
                    tooltip: 'Refresh logs',
                  ),
                ],
              ),
            ],
          ),
        ),
        // Logs List
        Expanded(
          child: Container(
            color: const Color(0xFF101010),
            padding: const EdgeInsets.all(12),
            child: ListView.builder(
              itemCount: logs.length,
              itemBuilder: (context, index) {
                final line = logs[index];
                
                // Color formatting
                Color textColor = Colors.white70;
                if (line.contains('[LSP Resolution]')) {
                  textColor = Colors.lightBlueAccent;
                } else if (line.contains('[LSP Spawn]')) {
                  textColor = Colors.greenAccent;
                } else if (line.contains('[LSP Spawn Error]') || line.contains('ERROR') || line.contains('❌') || line.contains('⚠️')) {
                  textColor = Colors.redAccent;
                } else if (line.contains('[STDERR]') || line.contains('[API STDERR]')) {
                  textColor = Colors.amber;
                }
                
                return Padding(
                  padding: const EdgeInsets.symmetric(vertical: 2.0),
                  child: Text(
                    line,
                    style: TextStyle(
                      color: textColor,
                      fontFamily: 'monospace',
                      fontSize: 11,
                    ),
                  ),
                );
              },
            ),
          ),
        ),
      ],
    );
  }
}
