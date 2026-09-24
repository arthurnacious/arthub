import '../../../core/network/api_client.dart';
import '../../../core/storage/token_storage.dart';
import 'user.dart';

/// Wraps the /auth endpoints - see backend/src/Modules/Auth/AuthController.php.
/// AuthBloc is the only thing that should call this; nothing here knows
/// about Bloc state.
class AuthRepository {
  AuthRepository({required ApiClient apiClient, required TokenStorage tokenStorage})
      : _apiClient = apiClient,
        _tokenStorage = tokenStorage;

  final ApiClient _apiClient;
  final TokenStorage _tokenStorage;

  Future<User> login(String email, String password) async {
    final body = await _apiClient.post('/auth/login', body: {
      'email': email,
      'password': password,
    });
    await _tokenStorage.write(body['token'] as String);

    return me();
  }

  /// Whether a token is stored at all - checked at app start before
  /// trusting it (see AuthBloc's AuthCheckRequested handler), since a
  /// stored token can have expired or been revoked server-side.
  Future<bool> hasStoredToken() async => await _tokenStorage.read() != null;

  Future<User> me() async {
    final body = await _apiClient.get('/auth/me');
    return User.fromJson(body);
  }

  Future<void> logout() => _tokenStorage.clear();
}
