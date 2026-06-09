import type { ReactNode } from "react";
import { Sidebar } from "./Sidebar";

interface LayoutProps {
  children: ReactNode;
  title?: string;
}

export function Layout({ children, title }: LayoutProps) {
  return (
    <div className="shell">
      <Sidebar />
      <div className="content-area">
        {title && (
          <header className="topnav">
            <span className="topnav-title">{title}</span>
            <span className="topnav-spacer" />
            <span className="topnav-badge">running</span>
          </header>
        )}
        <main className="page-content">{children}</main>
      </div>
    </div>
  );
}
