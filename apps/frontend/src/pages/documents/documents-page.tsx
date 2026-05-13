import { Button, Icons, Page, PageContent, PageHeader } from "@mizan/ui";
import { Link } from "react-router-dom";

// Documents page — landing surface for the Document Vault. The encrypted
// store, document jobs, extraction adapter, citations, and review queue
// are implemented in Phase 2 (docs/mizan-smart-plan/PLAN.md prompts 10–14).
//
// Until then this page surfaces what already exists today: source
// documents attached to imports live alongside the import history in
// Activities, and asset-level notes/files live on each asset profile.
// No fake document rows are rendered.

export default function DocumentsPage() {
  return (
    <Page>
      <PageHeader heading="Documents" text="Statements, factsheets, and source files." />
      <PageContent>
        <div
          data-testid="documents-empty"
          className="rounded-lg border border-dashed bg-muted/30 px-6 py-12 text-center"
        >
          <Icons.FileText className="text-muted-foreground mx-auto size-10" aria-hidden="true" />
          <p className="mt-4 text-base font-medium">Document Vault is being built</p>
          <p className="text-muted-foreground mx-auto mt-2 max-w-prose text-sm">
            The encrypted Document Vault, extraction, citations, and review queue land in
            Phase 2 of this branch. Today, source files attached to imports live in
            Activities, and per-asset documents live on each asset’s profile.
          </p>
          <div className="mt-6 flex flex-wrap justify-center gap-2">
            <Button asChild variant="secondary">
              <Link to="/activities">Open Activities</Link>
            </Button>
            <Button asChild variant="ghost">
              <Link to="/holdings">View Holdings</Link>
            </Button>
          </div>
        </div>
      </PageContent>
    </Page>
  );
}
