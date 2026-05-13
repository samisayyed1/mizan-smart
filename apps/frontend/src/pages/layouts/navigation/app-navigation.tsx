import { getDynamicNavItems, subscribeToNavigationUpdates } from "@/addons/addons-runtime-context";
import { Icons } from "@mizan/ui/components/ui/icons";
import { useEffect, useState } from "react";

export interface NavLink {
  title: string;
  href: string;
  icon?: React.ReactNode;
  keywords?: string[];
  label?: string; // Optional descriptive label for launcher/search
}

export interface NavigationProps {
  primary: NavLink[];
  secondary?: NavLink[];
  addons?: NavLink[];
}

// mizan-smart senior-friendly navigation.
//
// Primary navigation matches docs/mizan-smart-plan/PLAN.md Prompt 2:
//   Home, Portfolio, Documents, Reports, Inbox, Settings.
//
// All existing routes (Activities, Insights, Performance, Goals, Assistant,
// Connect) remain reachable through secondary navigation and direct URLs;
// nothing has been deleted.
const staticNavigation: NavigationProps = {
  primary: [
    {
      icon: <Icons.Home className="size-6" />,
      title: "Home",
      href: "/dashboard",
      keywords: ["home", "dashboard", "overview", "summary"],
      label: "Home",
    },
    {
      icon: <Icons.Holdings className="size-6" />,
      title: "Portfolio",
      href: "/holdings",
      keywords: ["portfolio", "holdings", "assets", "positions"],
      label: "Portfolio",
    },
    {
      icon: <Icons.FileText className="size-6" />,
      title: "Documents",
      href: "/documents",
      keywords: ["documents", "statements", "files", "vault"],
      label: "Documents",
    },
    {
      icon: <Icons.PieChart className="size-6" />,
      title: "Reports",
      href: "/reports",
      keywords: ["reports", "performance", "income", "tax"],
      label: "Reports",
    },
    {
      icon: <Icons.Inbox className="size-6" />,
      title: "Inbox",
      href: "/inbox",
      keywords: ["inbox", "alerts", "review", "attention"],
      label: "Inbox",
    },
    {
      icon: <Icons.Settings className="size-6" />,
      title: "Settings",
      href: "/settings",
      keywords: ["settings", "preferences", "config", "configuration"],
      label: "Settings",
    },
  ],
  secondary: [
    {
      icon: <Icons.Activity className="size-6" />,
      title: "Activities",
      href: "/activities",
      keywords: ["transactions", "trades", "history"],
      label: "Activities",
    },
    {
      icon: <Icons.Insight className="size-6" />,
      title: "Insights",
      href: "/insights",
      keywords: ["insights", "analytics", "breakdown"],
      label: "Insights",
    },
    {
      icon: <Icons.Goals className="size-6" />,
      title: "Goals",
      href: "/goals",
      keywords: ["goals", "fire", "retire", "retirement", "savings", "planner"],
      label: "Goals",
    },
    {
      icon: <Icons.Sparkles className="size-6" />,
      title: "Assistant",
      href: "/assistant",
      keywords: ["ai", "assistant", "chat", "help", "ask"],
      label: "AI Assistant",
    },
    {
      icon: <Icons.Link className="size-6" />,
      title: "Connect",
      href: "/connect",
      keywords: ["sync", "broker", "device", "supabase", "cloud", "account"],
      label: "Mizan Connect",
    },
  ],
};

export function useNavigation() {
  const [dynamicItems, setDynamicItems] = useState<NavigationProps["addons"]>([]);

  // Subscribe to navigation updates from addons
  useEffect(() => {
    const updateDynamicItems = () => {
      const itemsFromRuntime = getDynamicNavItems();
      setDynamicItems(itemsFromRuntime);
    };

    // Initial load
    updateDynamicItems();

    // Subscribe to updates
    const unsubscribe = subscribeToNavigationUpdates(updateDynamicItems);

    return () => {
      unsubscribe();
    };
  }, []);

  // Combine static navigation items with addons grouped separately.
  // Hide desktop-only features (FIRE Planner) in web mode.
  const primary = staticNavigation.primary;
  const navigation: NavigationProps = {
    primary,
    secondary: staticNavigation.secondary,
    addons: dynamicItems,
  };

  return navigation;
}

export function isPathActive(pathname: string, href: string): boolean {
  if (!href) {
    return false;
  }

  const ensureLeadingSlash = href.startsWith("/") ? href : `/${href}`;
  const normalize = (value: string) => {
    if (value.length > 1 && value.endsWith("/")) {
      return value.slice(0, -1);
    }
    return value;
  };

  const normalizedHref = normalize(ensureLeadingSlash);
  const normalizedPath = normalize(pathname);

  if (normalizedHref === "/") {
    return normalizedPath === "/";
  }

  // Dashboard and Net Worth are grouped together
  if (normalizedHref === "/dashboard") {
    return (
      normalizedPath === "/" || normalizedPath === "/dashboard" || normalizedPath === "/net-worth"
    );
  }

  return normalizedPath === normalizedHref || normalizedPath.startsWith(`${normalizedHref}/`);
}
