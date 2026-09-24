import 'package:flutter/material.dart';

import '../theme/app_colors.dart';

/// The "ARTHUB" wordmark - "ART" plain, "HUB" on a filled accent pill,
/// matching frontend/src/components/Logo.tsx. The web version also has a
/// blinking text cursor after it; skipped here to keep this scaffold's
/// first widget simple.
class Logo extends StatelessWidget {
  const Logo({super.key, this.fontSize = 20});

  final double fontSize;

  @override
  Widget build(BuildContext context) {
    final style = TextStyle(
      fontSize: fontSize,
      fontWeight: FontWeight.bold,
      letterSpacing: 2,
    );

    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        Text('ART', style: style.copyWith(color: AppColors.text)),
        const SizedBox(width: 2),
        Container(
          padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 1),
          decoration: BoxDecoration(
            color: AppColors.accent,
            borderRadius: BorderRadius.circular(3),
          ),
          child: Text('HUB', style: style.copyWith(color: AppColors.background)),
        ),
      ],
    );
  }
}
