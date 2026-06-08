import { NextResponse } from "next/server";

interface ContactPayload {
  name: string;
  email: string;
  company: string;
  role: string;
  useCase: string;
  interest: string;
  size?: string;
  timeline?: string;
}

export async function POST(request: Request) {
  let body: ContactPayload;

  try {
    body = await request.json();
  } catch {
    return NextResponse.json({ error: "Invalid JSON" }, { status: 400 });
  }

  const { name, email, company, role, useCase, interest, size, timeline } =
    body;

  if (!name || !email || !company || !role || !useCase || !interest) {
    return NextResponse.json(
      { error: "Missing required fields" },
      { status: 400 }
    );
  }

  const timestamp = new Date().toISOString();

  // ── Resend (email notification) ──
  if (process.env.RESEND_API_KEY) {
    try {
      const { Resend } = await import("resend");
      const resend = new Resend(process.env.RESEND_API_KEY);

      const fromAddress =
        process.env.RESEND_FROM || "UCM <onboarding@resend.dev>";
      const toAddress =
        process.env.NOTIFICATION_EMAIL || "me@paritoshkulkarni.space";

      await resend.emails.send({
        from: fromAddress,
        to: toAddress,
        subject: `[UCM Lead] ${name} @ ${company} — ${interest}`,
        html: `
          <h2>New Consulting Inquiry</h2>
          <table style="border-collapse:collapse">
            <tr><td style="padding:4px 12px 4px 0;font-weight:bold">Name</td><td>${name}</td></tr>
            <tr><td style="padding:4px 12px 4px 0;font-weight:bold">Email</td><td>${email}</td></tr>
            <tr><td style="padding:4px 12px 4px 0;font-weight:bold">Company</td><td>${company}</td></tr>
            <tr><td style="padding:4px 12px 4px 0;font-weight:bold">Role</td><td>${role}</td></tr>
            <tr><td style="padding:4px 12px 4px 0;font-weight:bold">Interest</td><td>${interest}</td></tr>
            <tr><td style="padding:4px 12px 4px 0;font-weight:bold">Size</td><td>${size || "—"}</td></tr>
            <tr><td style="padding:4px 12px 4px 0;font-weight:bold">Timeline</td><td>${timeline || "—"}</td></tr>
            <tr><td style="padding:4px 12px 4px 0;font-weight:bold">Submitted</td><td>${timestamp}</td></tr>
          </table>
          <h3>Use Case</h3>
          <p>${useCase}</p>
        `,
      });
    } catch (err) {
      console.error("[Resend] Failed to send email:", err);
    }
  }

  // ── Airtable (CRM record) ──
  if (
    process.env.AIRTABLE_TOKEN &&
    process.env.AIRTABLE_BASE_ID &&
    process.env.AIRTABLE_TABLE_NAME
  ) {
    try {
      const airtableUrl = `https://api.airtable.com/v0/${process.env.AIRTABLE_BASE_ID}/${encodeURIComponent(process.env.AIRTABLE_TABLE_NAME)}`;

      await fetch(airtableUrl, {
        method: "POST",
        headers: {
          Authorization: `Bearer ${process.env.AIRTABLE_TOKEN}`,
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          records: [
            {
              fields: {
                Name: name,
                "Work Email": email,
                Company: company,
                Role: role,
                "Use Case": useCase,
                Interest: interest,
                "Company Size": size || "",
                Timeline: timeline || "",
                Status: "New",
                "Source / UTM":
                  request.headers.get("referer") || "direct",
              },
            },
          ],
        }),
      });
    } catch (err) {
      console.error("[Airtable] Failed to create record:", err);
    }
  }

  // Always log to server console as fallback
  console.log("[Contact]", { name, email, company, role, interest, timestamp });

  return NextResponse.json({ success: true });
}
