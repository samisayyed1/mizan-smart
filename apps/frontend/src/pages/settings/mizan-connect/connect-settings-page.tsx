import { ComingSoonCard } from "@/components/coming-soon-card";
import { ConnectedView } from "@/features/mizan-connect/components/connected-view";
import { LoginForm } from "@/features/mizan-connect/components/login-form";
import { useMizanConnect } from "@/features/mizan-connect/providers/mizan-connect-provider";
import { Separator } from "@mizan/ui/components/ui/separator";
import { useEffect } from "react";
import { useLocation } from "react-router-dom";
import { SettingsHeader } from "../settings-header";

/**
 * Mizan Connect settings tab.
 *
 * Thin wrapper around existing components — no auth or fetch logic of its
 * own. Conditionally renders {@link LoginForm} when signed out and
 * {@link ConnectedView} when signed in. When the Connect feature flag is off
 * (no `.env` configuration), falls back to a placeholder so the tab is still
 * reachable but doesn't expose disabled controls.
 *
 * v3.3.3 instrumentation: logs to console on mount + renders a unique
 * runtime marker at the top of the page so we can confirm at install
 * time which path the user is actually on. The marker is removed in
 * the next release once the routing is verified working in production.
 */
export default function ConnectSettingsPage() {
  const { isEnabled, isConnected, isInitializing } = useMizanConnect();
  const location = useLocation();

  useEffect(() => {
    // eslint-disable-next-line no-console
    console.log("[mizan-connect] ConnectSettingsPage mounted at", location.pathname, {
      isEnabled,
      isConnected,
      isInitializing,
    });
  }, [location.pathname, isEnabled, isConnected, isInitializing]);

  return (
    <div className="space-y-6">
      <div
        data-testid="mizan-connect-route-marker"
        className="rounded-md border border-amber-400/40 bg-amber-400/10 px-3 py-2 font-mono text-xs text-amber-700 dark:text-amber-300"
      >
        v3.3.3 · /settings/connect · isEnabled={String(isEnabled)} · isConnected=
        {String(isConnected)} · pathname={location.pathname}
      </div>
      <SettingsHeader
        heading="Mizan Connect"
        text="Sign in to enable cross-device sync and the upcoming brokerage integrations."
      />
      <Separator />
      {!isEnabled ? (
        <ComingSoonCard
          title="Mizan Connect is disabled in this build"
          message="Set CONNECT_AUTH_URL and CONNECT_AUTH_PUBLISHABLE_KEY in your .env to enable the sign-in flow."
          detail="Mizan still works fully offline without these."
        />
      ) : isInitializing ? null : isConnected ? (
        <ConnectedView />
      ) : (
        <LoginForm />
      )}
    </div>
  );
}
