import 'package:go_router/go_router.dart';

import '../../features/auth/bloc/auth_bloc.dart';
import '../../features/auth/presentation/login_page.dart';
import '../../features/shell/presentation/app_shell_page.dart';
import '../../features/shell/presentation/splash_page.dart';
import 'go_router_refresh_stream.dart';

/// The mobile equivalent of the web frontend's per-route beforeLoad
/// guards (see e.g. frontend/src/routes/clients.tsx) - one redirect
/// callback, driven by AuthBloc, instead of one guard per route, since
/// go_router centralizes it that way.
GoRouter buildAppRouter(AuthBloc authBloc) {
  return GoRouter(
    initialLocation: '/',
    refreshListenable: GoRouterRefreshStream(authBloc.stream),
    redirect: (context, state) {
      final authState = authBloc.state;
      final loggingIn = state.matchedLocation == '/login';

      if (authState is AuthInitial || authState is AuthLoading) {
        return null; // stay put - '/' renders SplashPage until this resolves
      }

      final authenticated = authState is AuthAuthenticated;
      if (!authenticated && !loggingIn) return '/login';
      if (authenticated && loggingIn) return '/';

      return null;
    },
    routes: [
      GoRoute(path: '/login', builder: (context, state) => const LoginPage()),
      GoRoute(
        path: '/',
        builder: (context, state) {
          final authState = authBloc.state;
          return authState is AuthAuthenticated ? const AppShellPage() : const SplashPage();
        },
      ),
    ],
  );
}
