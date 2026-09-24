import 'package:flutter/material.dart';

/// Same tokens as the web frontend's `:root` custom properties
/// (frontend/src/styles.css) - kept in sync by hand, since there's no
/// shared design-token source between the two apps yet. Default (green)
/// accent only for now; the web app's purple/orange/red theme switcher
/// isn't replicated here.
class AppColors {
  const AppColors._();

  static const accent = Color(0xFF4ADE80); // green-400
  static const accentDim = Color(0x334ADE80); // accent at 20% alpha

  static const background = Color(0xFF0A0A0A);
  static const panel = Color(0xFF111214);
  static const border = Color(0xFF27272A);
  static const text = Color(0xFFD4D4D8); // zinc-300
  static const textDim = Color(0xFF71717A); // zinc-500

  static const danger = Color(0xFFF87171); // red-400
  static const info = Color(0xFF60A5FA); // blue-400
  static const success = accent;
}
