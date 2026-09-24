import 'package:flutter/material.dart';

import '../theme/app_colors.dart';

/// A bordered, titled box - same building block as the web frontend's
/// Panel.tsx: a border, a title bar, padded content underneath. No
/// elevation/shadow, since this whole app avoids Material's default
/// elevated look in favor of the web app's flat terminal-panel style.
class Panel extends StatelessWidget {
  const Panel({super.key, required this.title, required this.child});

  final String title;
  final Widget child;

  @override
  Widget build(BuildContext context) {
    return DecoratedBox(
      decoration: BoxDecoration(
        color: AppColors.panel,
        border: Border.all(color: AppColors.border),
        borderRadius: BorderRadius.circular(4),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
            decoration: const BoxDecoration(
              border: Border(bottom: BorderSide(color: AppColors.border)),
            ),
            child: Text(
              title,
              style: const TextStyle(
                fontSize: 11,
                color: AppColors.textDim,
                letterSpacing: 0.5,
              ),
            ),
          ),
          Padding(padding: const EdgeInsets.all(16), child: child),
        ],
      ),
    );
  }
}
