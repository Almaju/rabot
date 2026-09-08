enum UserRole {
    Member,
    Admin,
    Guest { invited_by: UserId, since: DateTime },
}
