import 'package:flutter/material.dart';

import '../../../core/theme/app_colors.dart';
import '../data/client_input.dart';

/// Shared company/contact/email/status fields - used by both the "new
/// client" screen and the detail page's edit mode, same reuse as the web
/// frontend's ClientForm.tsx. Purely controlled: the parent owns
/// [value] and receives every keystroke/selection via [onChanged],
/// same pattern LoginPage's fields use, just multi-field.
class ClientFormFields extends StatelessWidget {
  const ClientFormFields({super.key, required this.value, required this.onChanged});

  final ClientInput value;
  final ValueChanged<ClientInput> onChanged;

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        _Field(
          label: 'company',
          initialValue: value.company,
          onChanged: (text) => onChanged(value.copyWith(company: text)),
        ),
        const SizedBox(height: 12),
        _Field(
          label: 'contact',
          initialValue: value.contact,
          onChanged: (text) => onChanged(value.copyWith(contact: text)),
        ),
        const SizedBox(height: 12),
        _Field(
          label: 'email',
          initialValue: value.email,
          keyboardType: TextInputType.emailAddress,
          onChanged: (text) => onChanged(value.copyWith(email: text)),
        ),
        const SizedBox(height: 12),
        Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const Text('status', style: TextStyle(fontSize: 11, color: AppColors.textDim)),
            const SizedBox(height: 4),
            DropdownButtonFormField<String>(
              initialValue: value.status,
              items: [
                for (final status in clientStatusOptions)
                  DropdownMenuItem(value: status, child: Text(status)),
              ],
              onChanged: (status) {
                if (status != null) onChanged(value.copyWith(status: status));
              },
            ),
          ],
        ),
      ],
    );
  }
}

class _Field extends StatelessWidget {
  const _Field({
    required this.label,
    required this.initialValue,
    required this.onChanged,
    this.keyboardType,
  });

  final String label;
  final String initialValue;
  final ValueChanged<String> onChanged;
  final TextInputType? keyboardType;

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(label, style: const TextStyle(fontSize: 11, color: AppColors.textDim)),
        const SizedBox(height: 4),
        TextFormField(
          initialValue: initialValue,
          keyboardType: keyboardType,
          onChanged: onChanged,
        ),
      ],
    );
  }
}
