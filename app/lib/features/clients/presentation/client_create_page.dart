import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/theme/app_colors.dart';
import '../../../core/widgets/panel.dart';
import '../bloc/client_create_bloc.dart';
import '../data/clients_repository.dart';
import 'client_form_fields.dart';

/// Matches the web frontend's "new client" modal (see
/// frontend/src/routes/clients.index.tsx) - a full screen here instead
/// of a modal, the more natural mobile pattern for a 4-field form. Pops
/// `true` on a successful create, so ClientsPage knows to refresh.
class ClientCreatePage extends StatelessWidget {
  const ClientCreatePage({super.key});

  @override
  Widget build(BuildContext context) {
    return BlocProvider(
      create: (context) => ClientCreateBloc(clientsRepository: context.read<ClientsRepository>()),
      child: Scaffold(
        appBar: AppBar(title: const Text('new client')),
        body: BlocConsumer<ClientCreateBloc, ClientCreateState>(
          listenWhen: (previous, current) => current.status == ClientCreateStatus.success,
          listener: (context, state) => Navigator.of(context).pop(true),
          builder: (context, state) {
            final submitting = state.status == ClientCreateStatus.submitting;

            return SingleChildScrollView(
              padding: const EdgeInsets.all(16),
              child: Panel(
                title: 'client',
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  children: [
                    ClientFormFields(
                      value: state.form,
                      onChanged: (form) =>
                          context.read<ClientCreateBloc>().add(ClientCreateFormChanged(form)),
                    ),
                    if (state.errorMessage != null) ...[
                      const SizedBox(height: 8),
                      Text(state.errorMessage!, style: const TextStyle(color: AppColors.danger, fontSize: 12)),
                    ],
                    const SizedBox(height: 16),
                    ElevatedButton(
                      onPressed: submitting || state.form.company.trim().isEmpty
                          ? null
                          : () => context.read<ClientCreateBloc>().add(const ClientCreateSubmitted()),
                      child: Text(submitting ? 'creating...' : 'create'),
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
