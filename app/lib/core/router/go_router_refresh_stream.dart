import 'dart:async';

import 'package:flutter/foundation.dart';

/// Adapts a Stream (here, AuthBloc's state stream) into the Listenable
/// GoRouter's `refreshListenable` wants, so a redirect() re-runs whenever
/// auth state changes - the standard go_router + bloc recipe.
class GoRouterRefreshStream extends ChangeNotifier {
  GoRouterRefreshStream(Stream<dynamic> stream) {
    notifyListeners();
    _subscription = stream.asBroadcastStream().listen((_) => notifyListeners());
  }

  late final StreamSubscription<dynamic> _subscription;

  @override
  void dispose() {
    _subscription.cancel();
    super.dispose();
  }
}
