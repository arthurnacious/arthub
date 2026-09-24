import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

import 'core/network/api_client.dart';
import 'core/router/app_router.dart';
import 'core/storage/token_storage.dart';
import 'core/theme/app_theme.dart';
import 'features/auth/bloc/auth_bloc.dart';
import 'features/auth/data/auth_repository.dart';
import 'features/clients/bloc/clients_bloc.dart';
import 'features/clients/data/clients_repository.dart';

void main() {
  runApp(const ArthubApp());
}

class ArthubApp extends StatefulWidget {
  const ArthubApp({super.key});

  @override
  State<ArthubApp> createState() => _ArthubAppState();
}

class _ArthubAppState extends State<ArthubApp> {
  late final ClientsRepository _clientsRepository;
  late final AuthBloc _authBloc;
  late final ClientsBloc _clientsBloc;
  late final GoRouter _router;

  @override
  void initState() {
    super.initState();

    final tokenStorage = TokenStorage();
    // onUnauthorized closes over `_authBloc` (assigned two lines below,
    // before this client is ever used) rather than needing it passed in
    // up front - a 401 from any request logs the whole app out, same
    // effect as the web frontend just losing its token and the next
    // route guard bouncing to /login.
    final apiClient = ApiClient(
      tokenStorage: tokenStorage,
      onUnauthorized: () => _authBloc.add(const AuthLogoutRequested()),
    );
    final authRepository = AuthRepository(apiClient: apiClient, tokenStorage: tokenStorage);
    _clientsRepository = ClientsRepository(apiClient: apiClient);

    _authBloc = AuthBloc(authRepository: authRepository)..add(const AuthCheckRequested());
    _clientsBloc = ClientsBloc(clientsRepository: _clientsRepository);
    _router = buildAppRouter(_authBloc);
  }

  @override
  void dispose() {
    _authBloc.close();
    _clientsBloc.close();
    _router.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return RepositoryProvider.value(
      // Exposed so per-screen blocs pushed via Navigator (client detail,
      // client create - see clients_page.dart) can build their own
      // ClientDetailBloc/ClientCreateBloc from it, without ArthubApp
      // needing to know those screens exist.
      value: _clientsRepository,
      child: MultiBlocProvider(
        providers: [
          BlocProvider.value(value: _authBloc),
          BlocProvider.value(value: _clientsBloc),
        ],
        child: MaterialApp.router(
          title: 'Arthub',
          debugShowCheckedModeBanner: false,
          theme: AppTheme.dark,
          darkTheme: AppTheme.dark,
          themeMode: ThemeMode.dark,
          routerConfig: _router,
        ),
      ),
    );
  }
}
