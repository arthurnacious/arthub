import 'package:flutter/material.dart';

import '../theme/app_colors.dart';
import 'panel.dart';

/// Placeholder body for a nav section this scaffold hasn't built out yet
/// (Projects/Quotes/Invoices/Activity/Profile) - same Panel chrome as
/// every real page, so the rest of the shell doesn't look unfinished
/// while a section itself is still a stub. Follow ClientsPage's
/// data/bloc/presentation split to fill one of these in for real.
class ComingSoonPage extends StatelessWidget {
  const ComingSoonPage({super.key, required this.section});

  final String section;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.all(16),
      child: Panel(
        title: section,
        child: const Text(
          '-- not built out yet, see ClientsPage for the pattern to follow --',
          style: TextStyle(color: AppColors.textDim, fontSize: 13),
        ),
      ),
    );
  }
}
