import { useMutation, useQueryClient } from "@tanstack/react-query";
import { toast } from "@mizan/ui/components/ui/use-toast";
import { createAccount, updateAccount, deleteAccount, logger } from "@/adapters";
import { QueryKeys } from "@/lib/query-keys";
interface UseAccountMutationsProps {
  onSuccess?: () => void;
}

export function useAccountMutations({ onSuccess = () => undefined }: UseAccountMutationsProps) {
  const queryClient = useQueryClient();

  const handleSuccess = (message?: string) => {
    onSuccess();
    if (message) {
      toast({ title: message, variant: "success" });
    }
  };

  // Backend errors here travel as plain strings from the Tauri command
  // (`Result<_, String>`). We surface the message verbatim in the toast
  // description — the previous "Please try again" fallback hid the real
  // failure ("FOREIGN KEY constraint failed", "database is locked", etc.)
  // and turned every delete error into a guessing game for both the user
  // and us. Truncate at 240 chars so a runaway error doesn't blow up the
  // toast layout.
  const describeError = (e: unknown): string | undefined => {
    const raw =
      typeof e === "string"
        ? e
        : e instanceof Error
          ? e.message
          : e && typeof e === "object" && "message" in e && typeof e.message === "string"
            ? e.message
            : undefined;
    if (!raw) return undefined;
    const trimmed = raw.trim();
    if (!trimmed) return undefined;
    return trimmed.length > 240 ? `${trimmed.slice(0, 237)}…` : trimmed;
  };

  const handleError = (action: string, e: unknown) => {
    const detail = describeError(e);
    toast({
      title: `Couldn't ${action} this account`,
      description: detail ?? "An unexpected error occurred. Please try again or report an issue.",
      variant: "destructive",
    });
  };

  const createAccountMutation = useMutation({
    mutationFn: createAccount,
    onSuccess: () => {
      handleSuccess("Account created successfully.");
      queryClient.invalidateQueries({ queryKey: [QueryKeys.ACCOUNTS] });
    },
    onError: (e) => {
      logger.error(`Error creating account: ${e}`);
      handleError("create", e);
    },
  });

  const updateAccountMutation = useMutation({
    mutationFn: updateAccount,
    onSuccess: () => {
      handleSuccess();
      queryClient.invalidateQueries({ queryKey: [QueryKeys.ACCOUNTS] });
    },
    onError: (e) => {
      logger.error(`Error updating account: ${e}`);
      handleError("update", e);
    },
  });

  const deleteAccountMutation = useMutation({
    mutationFn: deleteAccount,
    onSuccess: () => {
      handleSuccess();
      queryClient.invalidateQueries({ queryKey: [QueryKeys.ACCOUNTS] });
    },
    onError: (e) => {
      logger.error(`Error deleting account: ${e}`);
      handleError("delete", e);
    },
  });

  return {
    createAccountMutation,
    updateAccountMutation,
    deleteAccountMutation,
  };
}
