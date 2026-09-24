part of 'client_detail_bloc.dart';

sealed class ClientDetailEvent extends Equatable {
  const ClientDetailEvent();

  @override
  List<Object?> get props => [];
}

class ClientDetailRequested extends ClientDetailEvent {
  const ClientDetailRequested();
}

class ClientDetailEditToggled extends ClientDetailEvent {
  const ClientDetailEditToggled(this.editing);

  final bool editing;

  @override
  List<Object?> get props => [editing];
}

class ClientDetailFormChanged extends ClientDetailEvent {
  const ClientDetailFormChanged(this.form);

  final ClientInput form;

  @override
  List<Object?> get props => [form];
}

class ClientDetailSubmitted extends ClientDetailEvent {
  const ClientDetailSubmitted();
}

class ClientDetailDeleteRequested extends ClientDetailEvent {
  const ClientDetailDeleteRequested();
}
