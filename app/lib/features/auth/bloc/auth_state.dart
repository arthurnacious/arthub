part of 'auth_bloc.dart';

sealed class AuthState extends Equatable {
  const AuthState();

  @override
  List<Object?> get props => [];
}

/// Before AuthCheckRequested has resolved - the router shows a loading
/// screen rather than guessing at a destination (see app_router.dart).
class AuthInitial extends AuthState {
  const AuthInitial();
}

class AuthLoading extends AuthState {
  const AuthLoading();
}

class AuthAuthenticated extends AuthState {
  const AuthAuthenticated(this.user);

  final User user;

  @override
  List<Object?> get props => [user];
}

class AuthUnauthenticated extends AuthState {
  const AuthUnauthenticated({this.error});

  /// Set only when this state was reached because a login attempt
  /// failed - null on a plain logout or a first-run "no session yet".
  final String? error;

  @override
  List<Object?> get props => [error];
}
