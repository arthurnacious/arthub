import 'package:equatable/equatable.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/network/api_exception.dart';
import '../data/auth_repository.dart';
import '../data/user.dart';

part 'auth_event.dart';
part 'auth_state.dart';

/// App-wide session state. Everything that needs to know "are we logged
/// in, and as whom" reads this instead of touching AuthRepository/token
/// storage directly - notably app_router.dart's redirect guard, which is
/// the mobile equivalent of the web frontend's per-route beforeLoad
/// guards (see e.g. frontend/src/routes/clients.tsx).
class AuthBloc extends Bloc<AuthEvent, AuthState> {
  AuthBloc({required AuthRepository authRepository})
      : _authRepository = authRepository,
        super(const AuthInitial()) {
    on<AuthCheckRequested>(_onCheckRequested);
    on<AuthLoginRequested>(_onLoginRequested);
    on<AuthLogoutRequested>(_onLogoutRequested);
  }

  final AuthRepository _authRepository;

  Future<void> _onCheckRequested(AuthCheckRequested event, Emitter<AuthState> emit) async {
    if (!await _authRepository.hasStoredToken()) {
      emit(const AuthUnauthenticated());
      return;
    }

    try {
      final user = await _authRepository.me();
      emit(AuthAuthenticated(user));
    } on ApiException {
      // Stored token is missing/expired/invalid - same effect as never
      // having logged in.
      await _authRepository.logout();
      emit(const AuthUnauthenticated());
    }
  }

  Future<void> _onLoginRequested(AuthLoginRequested event, Emitter<AuthState> emit) async {
    emit(const AuthLoading());

    try {
      final user = await _authRepository.login(event.email, event.password);
      emit(AuthAuthenticated(user));
    } on ApiException catch (error) {
      emit(AuthUnauthenticated(error: error.message));
    }
  }

  Future<void> _onLogoutRequested(AuthLogoutRequested event, Emitter<AuthState> emit) async {
    await _authRepository.logout();
    emit(const AuthUnauthenticated());
  }
}
