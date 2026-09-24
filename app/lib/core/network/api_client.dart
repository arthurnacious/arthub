import 'package:dio/dio.dart';

import '../storage/token_storage.dart';
import 'api_exception.dart';

/// Base URL of the arthub backend - same idea as the web frontend's
/// VITE_API_URL (frontend/.env), but set at build/run time via
/// `--dart-define=API_BASE_URL=...` instead of an env file, since that's
/// how Flutter passes config into a compiled app.
///
/// Defaults to the dev machine's LAN IP (not localhost/127.0.0.1) so a
/// physical phone on the same network reaches it out of the box, same
/// reasoning as the frontend's own default - see arthub/backend/.env's
/// CORS_ALLOWED_ORIGIN comment. Two exceptions worth knowing when running
/// against a local backend instead of a real device:
///   - Android emulator: use `--dart-define=API_BASE_URL=http://10.0.2.2:9501`
///     (its special alias for the host machine's localhost).
///   - iOS simulator: `http://localhost:9501` works directly, since the
///     simulator shares the host's network stack.
const _defaultBaseUrl = 'http://192.168.0.170:9501';
const apiBaseUrl = String.fromEnvironment('API_BASE_URL', defaultValue: _defaultBaseUrl);

/// Thin Dio wrapper: attaches the bearer token from [TokenStorage] to
/// every request, and turns any non-2xx response into an [ApiException]
/// carrying the backend's own error message - the same normalization the
/// web frontend's request() helper does (frontend/src/lib/api/http.ts).
///
/// [onUnauthorized] fires once per 401, letting whoever constructs this
/// client (see main.dart) react at the app level (clear the session,
/// bounce to the login screen) without ApiClient depending on AuthBloc
/// directly.
class ApiClient {
  ApiClient({
    required TokenStorage tokenStorage,
    this.onUnauthorized,
    Dio? dio,
  })  : _tokenStorage = tokenStorage,
        _dio = dio ??
            Dio(BaseOptions(
              baseUrl: apiBaseUrl,
              connectTimeout: const Duration(seconds: 10),
              receiveTimeout: const Duration(seconds: 10),
            )) {
    _dio.interceptors.add(
      InterceptorsWrapper(
        onRequest: (options, handler) async {
          final token = await _tokenStorage.read();
          if (token != null) {
            options.headers['Authorization'] = 'Bearer $token';
          }
          handler.next(options);
        },
      ),
    );
  }

  final Dio _dio;
  final TokenStorage _tokenStorage;
  final void Function()? onUnauthorized;

  Future<Map<String, dynamic>> get(String path, {Map<String, dynamic>? query}) =>
      _send(() => _dio.get(path, queryParameters: query));

  Future<Map<String, dynamic>> post(String path, {Object? body}) =>
      _send(() => _dio.post(path, data: body));

  Future<Map<String, dynamic>> patch(String path, {Object? body}) =>
      _send(() => _dio.patch(path, data: body));

  Future<Map<String, dynamic>> delete(String path) => _send(() => _dio.delete(path));

  Future<Map<String, dynamic>> _send(Future<Response<dynamic>> Function() request) async {
    try {
      final response = await request();
      final data = response.data;
      return data is Map<String, dynamic> ? data : <String, dynamic>{};
    } on DioException catch (error) {
      final statusCode = error.response?.statusCode ?? 0;
      final exception = ApiException.fromResponseBody(error.response?.data, statusCode);

      if (exception.isUnauthorized) onUnauthorized?.call();

      throw exception;
    }
  }
}
