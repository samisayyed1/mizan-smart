import { isWeb } from "@/adapters";
import { RootErrorBoundary } from "@/components/root-error-boundary";
import { AuthGate, AuthProvider } from "@/context/auth-context";
import { MizanConnectProvider } from "@/features/mizan-connect";
import { SettingsProvider } from "@/lib/settings-provider";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { TooltipProvider } from "@mizan/ui";
import { useState } from "react";
import { PrivacyProvider } from "./context/privacy-context";
import { LoginPage } from "./pages/auth/login-page";
import { AppRoutes } from "./routes";

function App() {
  const [queryClient] = useState(
    () =>
      new QueryClient({
        defaultOptions: {
          queries: {
            refetchOnWindowFocus: false,
            staleTime: 5 * 60 * 1000,
            retry: false,
          },
        },
      }),
  );

  const isWebEnv = isWeb;

  // Make QueryClient available globally for addons
  window.__mizan_query_client__ = queryClient;

  const routedContent = isWebEnv ? (
    <AuthGate fallback={<LoginPage />}>
      <AppRoutes />
    </AuthGate>
  ) : (
    <AppRoutes />
  );

  return (
    // Top-level error boundary OUTSIDE every provider so even a
    // provider that throws during initialisation (auth, query client,
    // settings) renders the recovery screen instead of a white page.
    <RootErrorBoundary>
      <QueryClientProvider client={queryClient}>
        <AuthProvider>
          <MizanConnectProvider>
            <PrivacyProvider>
              <SettingsProvider>
                <TooltipProvider>{routedContent}</TooltipProvider>
              </SettingsProvider>
            </PrivacyProvider>
          </MizanConnectProvider>
        </AuthProvider>
      </QueryClientProvider>
    </RootErrorBoundary>
  );
}

export default App;
