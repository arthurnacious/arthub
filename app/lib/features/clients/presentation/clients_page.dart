import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/theme/app_colors.dart';
import '../../../core/widgets/status_badge.dart';
import '../bloc/clients_bloc.dart';
import '../data/client.dart';

StatusTone _toneFor(String status) => switch (status) {
      'active' => StatusTone.success,
      'prospect' => StatusTone.info,
      _ => StatusTone.neutral,
    };

/// Matches frontend/src/routes/clients.index.tsx + ClientsTable.tsx, but
/// as an infinite-scroll list (the natural mobile pattern) rather than
/// numbered pages. Search is debounced client-side before reaching the
/// bloc, same 300ish-ms feel as the web app's useDebouncedValue hook
/// (frontend/src/hooks/use-debounced-value.ts).
class ClientsPage extends StatefulWidget {
  const ClientsPage({super.key});

  @override
  State<ClientsPage> createState() => _ClientsPageState();
}

class _ClientsPageState extends State<ClientsPage> {
  final _scrollController = ScrollController();
  final _searchController = TextEditingController();
  Timer? _debounce;

  @override
  void initState() {
    super.initState();
    context.read<ClientsBloc>().add(const ClientsRequested());
    _scrollController.addListener(_onScroll);
  }

  @override
  void dispose() {
    _debounce?.cancel();
    _scrollController.dispose();
    _searchController.dispose();
    super.dispose();
  }

  void _onScroll() {
    if (_scrollController.position.extentAfter < 300) {
      context.read<ClientsBloc>().add(const ClientsNextPageRequested());
    }
  }

  void _onSearchChanged(String value) {
    _debounce?.cancel();
    _debounce = Timer(const Duration(milliseconds: 300), () {
      context.read<ClientsBloc>().add(ClientsSearchChanged(value));
    });
  }

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        Padding(
          padding: const EdgeInsets.all(16),
          child: TextField(
            controller: _searchController,
            onChanged: _onSearchChanged,
            decoration: const InputDecoration(
              hintText: 'search clients...',
              prefixIcon: Icon(Icons.search, color: AppColors.textDim, size: 18),
              isDense: true,
            ),
          ),
        ),
        Expanded(
          child: BlocBuilder<ClientsBloc, ClientsState>(
            builder: (context, state) {
              if (state.status == ClientsStatus.loading) {
                return const Center(child: CircularProgressIndicator());
              }

              if (state.status == ClientsStatus.failure && state.clients.isEmpty) {
                return _ErrorView(
                  message: state.errorMessage ?? 'Something went wrong.',
                  onRetry: () => context.read<ClientsBloc>().add(const ClientsRequested()),
                );
              }

              if (state.clients.isEmpty) {
                return const Center(
                  child: Text('-- no clients match --', style: TextStyle(color: AppColors.textDim)),
                );
              }

              return RefreshIndicator(
                onRefresh: () async => context.read<ClientsBloc>().add(const ClientsRequested()),
                child: ListView.separated(
                  controller: _scrollController,
                  padding: const EdgeInsets.symmetric(horizontal: 16),
                  itemCount: state.clients.length + (state.hasNextPage ? 1 : 0),
                  separatorBuilder: (_, _) => const Divider(height: 1),
                  itemBuilder: (context, index) {
                    if (index >= state.clients.length) {
                      return const Padding(
                        padding: EdgeInsets.symmetric(vertical: 16),
                        child: Center(child: CircularProgressIndicator()),
                      );
                    }

                    return _ClientTile(client: state.clients[index]);
                  },
                ),
              );
            },
          ),
        ),
      ],
    );
  }
}

class _ClientTile extends StatelessWidget {
  const _ClientTile({required this.client});

  final Client client;

  @override
  Widget build(BuildContext context) {
    return ListTile(
      contentPadding: EdgeInsets.zero,
      title: Text(client.company, style: const TextStyle(color: AppColors.text)),
      subtitle: Text(
        [client.contact, client.email].whereType<String>().join(' · '),
        style: const TextStyle(color: AppColors.textDim, fontSize: 12),
      ),
      trailing: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          StatusBadge(label: client.status, tone: _toneFor(client.status)),
          const SizedBox(width: 8),
          Text('${client.projects}', style: const TextStyle(color: AppColors.textDim, fontSize: 12)),
        ],
      ),
      onTap: () => _showClientDetail(context, client),
    );
  }
}

void _showClientDetail(BuildContext context, Client client) {
  showModalBottomSheet<void>(
    context: context,
    backgroundColor: AppColors.panel,
    builder: (context) => Padding(
      padding: const EdgeInsets.all(20),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Expanded(
                child: Text(
                  client.company,
                  style: const TextStyle(fontSize: 16, fontWeight: FontWeight.bold, color: AppColors.text),
                ),
              ),
              StatusBadge(label: client.status, tone: _toneFor(client.status)),
            ],
          ),
          const SizedBox(height: 12),
          if (client.contact != null) _DetailRow(label: 'contact', value: client.contact!),
          if (client.email != null) _DetailRow(label: 'email', value: client.email!),
          _DetailRow(label: 'projects', value: '${client.projects}'),
        ],
      ),
    ),
  );
}

class _DetailRow extends StatelessWidget {
  const _DetailRow({required this.label, required this.value});

  final String label;
  final String value;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: Row(
        children: [
          SizedBox(
            width: 72,
            child: Text(label, style: const TextStyle(color: AppColors.textDim, fontSize: 12)),
          ),
          Expanded(child: Text(value, style: const TextStyle(color: AppColors.text, fontSize: 13))),
        ],
      ),
    );
  }
}

class _ErrorView extends StatelessWidget {
  const _ErrorView({required this.message, required this.onRetry});

  final String message;
  final VoidCallback onRetry;

  @override
  Widget build(BuildContext context) {
    return Center(
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          const Text(
            "COULDN'T LOAD CLIENTS",
            style: TextStyle(color: AppColors.danger, fontWeight: FontWeight.bold),
          ),
          const SizedBox(height: 8),
          Text(message, style: const TextStyle(color: AppColors.textDim, fontSize: 12)),
          const SizedBox(height: 16),
          OutlinedButton(onPressed: onRetry, child: const Text('retry')),
        ],
      ),
    );
  }
}
