import type { Metadata } from "next";
import { IBM_Plex_Mono, Schibsted_Grotesk } from "next/font/google";
import "./globals.css";

const sans = Schibsted_Grotesk({
  subsets: ["latin"],
  variable: "--font-sans",
});

const mono = IBM_Plex_Mono({
  subsets: ["latin"],
  variable: "--font-mono",
  weight: ["400", "500"],
});

export const metadata: Metadata = {
  title: "World ID Root Replicator",
  description:
    "Read-only observability surface for World ID root replication across EVM targets.",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body className={`${sans.variable} ${mono.variable}`}>
        <div className="site-background">
          <div className="site-background__mesh site-background__mesh--top" />
          <div className="site-background__mesh site-background__mesh--bottom" />
          <div className="site-background__grid" />
        </div>
        <div className="site-shell">{children}</div>
      </body>
    </html>
  );
}
