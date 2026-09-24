import 'package:flutter/material.dart';

import 'app_colors.dart';

const _fontFamily = 'JetBrains Mono';

/// Dark-only, terminal/CRT styled theme matching the web frontend
/// (frontend/src/styles.css + Panel.tsx/StatusBadge.tsx etc.) - JetBrains
/// Mono throughout, bordered panels instead of elevation/shadows, the
/// same accent green. The web app also has a light/scanline/matrix-rain
/// aesthetic layer that isn't attempted here; this covers the color and
/// typography system, which is what actually needs to stay consistent
/// for the two apps to feel like the same product.
///
/// The font is bundled as a local asset (see pubspec.yaml's `fonts:`
/// section, assets/fonts/) rather than fetched at runtime via
/// google_fonts - that package initially rendered every bit of text in
/// this app invisible on web, since the whole theme's fontFamily
/// depended on a network fetch that hadn't resolved yet by first paint.
/// A bundled font has no such race, and works offline besides.
class AppTheme {
  const AppTheme._();

  static ThemeData get dark {
    final textTheme = ThemeData.dark().textTheme.apply(
          fontFamily: _fontFamily,
          bodyColor: AppColors.text,
          displayColor: AppColors.text,
        );

    return ThemeData(
      useMaterial3: true,
      brightness: Brightness.dark,
      scaffoldBackgroundColor: AppColors.background,
      fontFamily: _fontFamily,
      textTheme: textTheme,
      colorScheme: const ColorScheme.dark(
        primary: AppColors.accent,
        secondary: AppColors.accent,
        surface: AppColors.panel,
        error: AppColors.danger,
        onPrimary: Colors.black,
        onSurface: AppColors.text,
      ),
      appBarTheme: const AppBarTheme(
        backgroundColor: AppColors.panel,
        foregroundColor: AppColors.text,
        elevation: 0,
        centerTitle: false,
        titleTextStyle: TextStyle(
          color: AppColors.accent,
          fontWeight: FontWeight.bold,
          fontSize: 16,
          letterSpacing: 0.5,
        ),
      ),
      drawerTheme: const DrawerThemeData(
        backgroundColor: AppColors.panel,
      ),
      cardTheme: CardThemeData(
        color: AppColors.panel,
        elevation: 0,
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(4),
          side: const BorderSide(color: AppColors.border),
        ),
      ),
      inputDecorationTheme: InputDecorationTheme(
        filled: true,
        fillColor: AppColors.background,
        hintStyle: const TextStyle(color: AppColors.textDim),
        border: OutlineInputBorder(
          borderRadius: BorderRadius.circular(4),
          borderSide: const BorderSide(color: AppColors.border),
        ),
        enabledBorder: OutlineInputBorder(
          borderRadius: BorderRadius.circular(4),
          borderSide: const BorderSide(color: AppColors.border),
        ),
        focusedBorder: OutlineInputBorder(
          borderRadius: BorderRadius.circular(4),
          borderSide: const BorderSide(color: AppColors.accent),
        ),
        errorBorder: OutlineInputBorder(
          borderRadius: BorderRadius.circular(4),
          borderSide: const BorderSide(color: AppColors.danger),
        ),
      ),
      elevatedButtonTheme: ElevatedButtonThemeData(
        style: ElevatedButton.styleFrom(
          backgroundColor: AppColors.accent,
          foregroundColor: Colors.black,
          textStyle: const TextStyle(fontWeight: FontWeight.bold),
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(4),
          ),
        ),
      ),
      outlinedButtonTheme: OutlinedButtonThemeData(
        style: OutlinedButton.styleFrom(
          foregroundColor: AppColors.textDim,
          side: const BorderSide(color: AppColors.border),
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(4),
          ),
        ),
      ),
      dividerTheme: const DividerThemeData(color: AppColors.border),
      progressIndicatorTheme: const ProgressIndicatorThemeData(
        color: AppColors.accent,
      ),
    );
  }
}
