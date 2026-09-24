import 'package:flutter/material.dart';

import '../theme/app_colors.dart';

/// Matches the web frontend's StatusBadge.tsx tones exactly, so the same
/// status string reads the same color on both apps.
enum StatusTone { neutral, info, success, danger }

class StatusBadge extends StatelessWidget {
  const StatusBadge({super.key, required this.label, required this.tone});

  final String label;
  final StatusTone tone;

  Color get _color => switch (tone) {
        StatusTone.neutral => AppColors.textDim,
        StatusTone.info => AppColors.info,
        StatusTone.success => AppColors.success,
        StatusTone.danger => AppColors.danger,
      };

  @override
  Widget build(BuildContext context) {
    final color = _color;

    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
      decoration: BoxDecoration(
        border: Border.all(color: color.withValues(alpha: 0.5)),
        borderRadius: BorderRadius.circular(3),
        color: color.withValues(alpha: 0.12),
      ),
      child: Text(
        label,
        style: TextStyle(
          color: color,
          fontSize: 11,
          fontWeight: FontWeight.w600,
        ),
      ),
    );
  }
}
