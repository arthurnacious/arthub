part of 'client_create_bloc.dart';

sealed class ClientCreateEvent extends Equatable {
  const ClientCreateEvent();

  @override
  List<Object?> get props => [];
}

class ClientCreateFormChanged extends ClientCreateEvent {
  const ClientCreateFormChanged(this.form);

  final ClientInput form;

  @override
  List<Object?> get props => [form];
}

class ClientCreateSubmitted extends ClientCreateEvent {
  const ClientCreateSubmitted();
}
