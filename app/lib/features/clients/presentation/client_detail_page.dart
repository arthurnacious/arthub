import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/theme/app_colors.dart';
import '../../../core/widgets/panel.dart';
import '../../../core/widgets/status_badge.dart';
import '../bloc/client_detail_bloc.dart';
import '../data/clients_repository.dart';
import 'client_form_fields.dart';
import 'clients_page.dart' show toneFor;

/// Matches frontend/src/routes/clients.$clientId.tsx: view fields, an
/// "edit" toggle that swaps the same Panel's body for ClientFormFields
/// in place, and delete (behind a confirm dialog). Pops `true` when the
/// client was changed or deleted, so ClientsPage knows to refresh its
/// list - see clients_page.dart.
class ClientDetailPage extends StatelessWidget {
  const ClientDetailPage({super.key, required this.clientId});

  final String clientId;

  @override
  Widget build(BuildContext context) {
    return BlocProvider(
      create: (context) => ClientDetailBloc(
        clientsRepository: context.read<ClientsRepository>(),
        clientId: clientId,
      )..add(const ClientDetailRequested()),
      child: const _ClientDetailView(),
    );
  }
}

class _ClientDetailView extends StatefulWidget {
  const _ClientDetailView();

  @override
  State<_ClientDetailView> createState() => _ClientDetailViewState();
}

class _ClientDetailViewState extends State<_ClientDetailView> {
  bool _changed = false;

  Future<void> _confirmDelete(BuildContext context) async {
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        backgroundColor: AppColors.panel,
        title: const Text('delete client'),
        content: const Text("This can't be undone."),
        actions: [
          TextButton(onPressed: () => Navigator.of(context).pop(false), child: const Text('cancel')),
          TextButton(
            onPressed: () => Navigator.of(context).pop(true),
            child: const Text('delete', style: TextStyle(color: AppColors.danger)),
          ),
        ],
      ),
    );

    if (confirmed == true && context.mounted) {
      context.read<ClientDetailBloc>().add(const ClientDetailDeleteRequested());
    }
  }

  @override
  Widget build(BuildContext context) {
    return PopScope(
      canPop: false,
      onPopInvokedWithResult: (didPop, result) {
        if (!didPop) Navigator.of(context).pop(_changed);
      },
      child: Scaffold(
        appBar: AppBar(title: const Text('client')),
        body: BlocConsumer<ClientDetailBloc, ClientDetailState>(
          listenWhen: (previous, current) =>
              current.status == ClientDetailStatus.deleted ||
              (previous.editing && !current.editing && current.status == ClientDetailStatus.loaded),
          listener: (context, state) {
            _changed = true;
            if (state.status == ClientDetailStatus.deleted) {
              Navigator.of(context).pop(true);
            }
          },
          builder: (context, state) {
            if (state.status == ClientDetailStatus.loading) {
              return const Center(child: CircularProgressIndicator());
            }

            if (state.status == ClientDetailStatus.failure && state.client == null) {
              return Center(
                child: Text(
                  state.errorMessage ?? "Couldn't load this client.",
                  style: const TextStyle(color: AppColors.danger),
                ),
              );
            }

            final client = state.client!;
            final saving = state.status == ClientDetailStatus.saving;
            final deleting = state.status == ClientDetailStatus.deleting;

            return SingleChildScrollView(
              padding: const EdgeInsets.all(16),
              child: Panel(
                title: state.editing ? 'edit client' : 'client',
                child: state.editing
                    ? Column(
                        crossAxisAlignment: CrossAxisAlignment.stretch,
                        children: [
                          ClientFormFields(
                            value: state.form,
                            onChanged: (form) =>
                                context.read<ClientDetailBloc>().add(ClientDetailFormChanged(form)),
                          ),
                          if (state.errorMessage != null) ...[
                            const SizedBox(height: 8),
                            Text(state.errorMessage!, style: const TextStyle(color: AppColors.danger, fontSize: 12)),
                          ],
                          const SizedBox(height: 16),
                          Row(
                            children: [
                              Expanded(
                                child: ElevatedButton(
                                  onPressed: saving
                                      ? null
                                      : () => context.read<ClientDetailBloc>().add(const ClientDetailSubmitted()),
                                  child: Text(saving ? 'saving...' : 'save'),
                                ),
                              ),
                              const SizedBox(width: 8),
                              OutlinedButton(
                                onPressed: saving
                                    ? null
                                    : () => context.read<ClientDetailBloc>().add(const ClientDetailEditToggled(false)),
                                child: const Text('cancel'),
                              ),
                            ],
                          ),
                        ],
                      )
                    : Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Row(
                            children: [
                              Expanded(
                                child: Text(
                                  client.company,
                                  style: const TextStyle(
                                    fontSize: 16,
                                    fontWeight: FontWeight.bold,
                                    color: AppColors.text,
                                  ),
                                ),
                              ),
                              StatusBadge(label: client.status, tone: toneFor(client.status)),
                            ],
                          ),
                          const SizedBox(height: 16),
                          if (client.contact != null && client.contact!.isNotEmpty)
                            _Row(label: 'contact', value: client.contact!),
                          if (client.email != null && client.email!.isNotEmpty)
                            _Row(label: 'email', value: client.email!),
                          _Row(label: 'projects', value: '${client.projects}'),
                          const SizedBox(height: 16),
                          Row(
                            children: [
                              OutlinedButton(
                                onPressed: () =>
                                    context.read<ClientDetailBloc>().add(const ClientDetailEditToggled(true)),
                                child: const Text('edit'),
                              ),
                              const SizedBox(width: 8),
                              OutlinedButton(
                                onPressed: deleting ? null : () => _confirmDelete(context),
                                style: OutlinedButton.styleFrom(foregroundColor: AppColors.danger),
                                child: Text(deleting ? 'deleting...' : 'delete'),
                              ),
                            ],
                          ),
                        ],
                      ),
              ),
            );
          },
        ),
      ),
    );
  }
}

class _Row extends StatelessWidget {
  const _Row({required this.label, required this.value});

  final String label;
  final String value;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: Row(
        children: [
          SizedBox(width: 72, child: Text(label, style: const TextStyle(color: AppColors.textDim, fontSize: 12))),
          Expanded(child: Text(value, style: const TextStyle(color: AppColors.text, fontSize: 13))),
        ],
      ),
    );
  }
}
