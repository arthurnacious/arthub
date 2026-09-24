import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/theme/app_colors.dart';
import '../../../core/widgets/logo.dart';
import '../../../core/widgets/panel.dart';
import '../bloc/auth_bloc.dart';

/// Matches frontend/src/routes/login.tsx: centered card, logo above a
/// "login" Panel, plain email/password fields. AuthBloc does the actual
/// request (via AuthLoginRequested) - this widget only reads its state to
/// show a spinner/error, same separation of concerns as every other
/// feature here.
class LoginPage extends StatefulWidget {
  const LoginPage({super.key});

  @override
  State<LoginPage> createState() => _LoginPageState();
}

class _LoginPageState extends State<LoginPage> {
  final _formKey = GlobalKey<FormState>();
  final _emailController = TextEditingController();
  final _passwordController = TextEditingController();

  @override
  void dispose() {
    _emailController.dispose();
    _passwordController.dispose();
    super.dispose();
  }

  void _submit() {
    if (!_formKey.currentState!.validate()) return;

    context.read<AuthBloc>().add(AuthLoginRequested(
          email: _emailController.text.trim(),
          password: _passwordController.text,
        ));
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: Center(
        child: SingleChildScrollView(
          padding: const EdgeInsets.all(24),
          child: ConstrainedBox(
            constraints: const BoxConstraints(maxWidth: 360),
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                const Logo(fontSize: 24),
                const SizedBox(height: 24),
                Panel(
                  title: 'login',
                  child: BlocBuilder<AuthBloc, AuthState>(
                    builder: (context, state) {
                      final submitting = state is AuthLoading;
                      final error = state is AuthUnauthenticated ? state.error : null;

                      return Form(
                        key: _formKey,
                        child: Column(
                          crossAxisAlignment: CrossAxisAlignment.stretch,
                          children: [
                            _FieldLabel(
                              label: 'email',
                              child: TextFormField(
                                controller: _emailController,
                                keyboardType: TextInputType.emailAddress,
                                textInputAction: TextInputAction.next,
                                enabled: !submitting,
                                validator: (value) =>
                                    (value == null || value.trim().isEmpty) ? 'Required' : null,
                              ),
                            ),
                            const SizedBox(height: 12),
                            _FieldLabel(
                              label: 'password',
                              child: TextFormField(
                                controller: _passwordController,
                                obscureText: true,
                                textInputAction: TextInputAction.done,
                                enabled: !submitting,
                                onFieldSubmitted: (_) => _submit(),
                                validator: (value) =>
                                    (value == null || value.isEmpty) ? 'Required' : null,
                              ),
                            ),
                            if (error != null) ...[
                              const SizedBox(height: 12),
                              Text(
                                error,
                                style: const TextStyle(color: AppColors.danger, fontSize: 12),
                              ),
                            ],
                            const SizedBox(height: 16),
                            ElevatedButton(
                              onPressed: submitting ? null : _submit,
                              child: Text(submitting ? 'logging in...' : 'log in'),
                            ),
                          ],
                        ),
                      );
                    },
                  ),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}

class _FieldLabel extends StatelessWidget {
  const _FieldLabel({required this.label, required this.child});

  final String label;
  final Widget child;

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(label, style: const TextStyle(fontSize: 11, color: AppColors.textDim)),
        const SizedBox(height: 4),
        child,
      ],
    );
  }
}
