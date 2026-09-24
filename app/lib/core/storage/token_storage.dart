import 'package:flutter_secure_storage/flutter_secure_storage.dart';

/// Thin wrapper around flutter_secure_storage - the mobile equivalent of
/// the web frontend's authStore (which keeps the JWT in localStorage).
/// Kept as its own class (rather than calling FlutterSecureStorage
/// directly from AuthRepository) so it's the one place that knows the
/// storage key and the one seam tests can fake.
class TokenStorage {
  TokenStorage({FlutterSecureStorage? storage})
      : _storage = storage ?? const FlutterSecureStorage();

  static const _tokenKey = 'auth_token';

  final FlutterSecureStorage _storage;

  Future<String?> read() => _storage.read(key: _tokenKey);

  Future<void> write(String token) => _storage.write(key: _tokenKey, value: token);

  Future<void> clear() => _storage.delete(key: _tokenKey);
}
