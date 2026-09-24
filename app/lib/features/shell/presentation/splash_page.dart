import 'package:flutter/material.dart';

import '../../../core/widgets/logo.dart';

/// Shown for the brief window between app start and AuthBloc resolving
/// AuthCheckRequested (is there a valid stored session or not) - see
/// app_router.dart's redirect logic.
class SplashPage extends StatelessWidget {
  const SplashPage({super.key});

  @override
  Widget build(BuildContext context) {
    return const Scaffold(
      body: Center(child: Logo(fontSize: 24)),
    );
  }
}
