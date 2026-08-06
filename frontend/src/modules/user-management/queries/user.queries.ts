import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { container } from "@/app/app.module";
import { SERVICE_NAMES, type IUserManagementService } from "@/commons/di/registry";

import type { DatabaseUser, Privilege } from "../types/user.types";

const USER_KEYS = {
  list: (connectionId: string) => ["users", connectionId] as const,
  privileges: (connectionId: string, roleName: string) =>
    ["user-privileges", connectionId, roleName] as const,
};

function getUserService() {
  return container.resolve<IUserManagementService>(
    SERVICE_NAMES.USER_MANAGEMENT_SERVICE,
  );
}

export function useListUsers(connectionId: string | null) {
  return useQuery({
    queryKey: USER_KEYS.list(connectionId ?? ""),
    queryFn: () =>
      getUserService().listUsers(connectionId!) as Promise<DatabaseUser[]>,
    enabled: !!connectionId,
  });
}

export function useCreateRole(connectionId: string | null) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ name, login }: { name: string; login: boolean }) =>
      getUserService().createRole(connectionId!, name, login),
    onSuccess: () => {
      if (connectionId) {
        qc.invalidateQueries({ queryKey: USER_KEYS.list(connectionId) });
      }
    },
  });
}

export function useDropRole(connectionId: string | null) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (name: string) => getUserService().dropRole(connectionId!, name),
    onSuccess: () => {
      if (connectionId) {
        qc.invalidateQueries({ queryKey: USER_KEYS.list(connectionId) });
      }
    },
  });
}

export function useListPrivileges(
  connectionId: string | null,
  roleName: string | null,
) {
  return useQuery({
    queryKey: USER_KEYS.privileges(connectionId ?? "", roleName ?? ""),
    queryFn: () =>
      getUserService().listPrivileges(connectionId!, roleName!) as Promise<Privilege[]>,
    enabled: !!connectionId && !!roleName,
  });
}

export function useGrantPrivilege(
  connectionId: string | null,
  roleName: string | null,
) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({
      schema,
      table,
      privilege,
    }: {
      schema: string;
      table: string;
      privilege: string;
    }) =>
      getUserService().grantPrivilege(
        connectionId!,
        roleName!,
        schema,
        table,
        privilege,
      ),
    onSuccess: () => {
      if (connectionId && roleName) {
        qc.invalidateQueries({
          queryKey: USER_KEYS.privileges(connectionId, roleName),
        });
      }
    },
  });
}

export function useRevokePrivilege(
  connectionId: string | null,
  roleName: string | null,
) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({
      schema,
      table,
      privilege,
    }: {
      schema: string;
      table: string;
      privilege: string;
    }) =>
      getUserService().revokePrivilege(
        connectionId!,
        roleName!,
        schema,
        table,
        privilege,
      ),
    onSuccess: () => {
      if (connectionId && roleName) {
        qc.invalidateQueries({
          queryKey: USER_KEYS.privileges(connectionId, roleName),
        });
      }
    },
  });
}
