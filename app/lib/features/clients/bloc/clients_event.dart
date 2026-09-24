part of 'clients_bloc.dart';

sealed class ClientsEvent extends Equatable {
  const ClientsEvent();

  @override
  List<Object?> get props => [];
}

/// Loads (or reloads, on pull-to-refresh) page 1 with the current search
/// term.
class ClientsRequested extends ClientsEvent {
  const ClientsRequested();
}

/// Debounced in the UI (see clients_page.dart) before being dispatched -
/// the bloc itself just reloads from page 1 whenever this fires.
class ClientsSearchChanged extends ClientsEvent {
  const ClientsSearchChanged(this.query);

  final String query;

  @override
  List<Object?> get props => [query];
}

/// Fired when the list is scrolled near its end - appends the next page
/// rather than replacing the list.
class ClientsNextPageRequested extends ClientsEvent {
  const ClientsNextPageRequested();
}
