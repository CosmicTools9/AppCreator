export function evaluateExpr(expr: string, context: Record<string, unknown>): { result: boolean; error: string | null } {
  try {
    if (!expr.trim()) return { result: true, error: null };
    const tokens = tokenize(expr);
    const ast = new Parser(tokens).parse();
    return { result: !!evalNode(ast, context), error: null };
  } catch (e) { return { result: false, error: (e as Error).message }; }
}
type T = { type: string; value: string };
function tokenize(s: string): T[] {
  const r: T[] = []; let i = 0;
  while (i < s.length) {
    const c = s[i]!;
    if (c === ' ' || c === '\t' || c === '\n') { i++; continue; }
    if ('()[]!,><=+-*/%&|!'.includes(c)) { r.push({ type: c, value: c }); i++; continue; }
    if (c === '"' || c === "'") { let v = ''; i++; while (i < s.length && s[i] !== c) { v += s[i]!; i++; } i++; r.push({ type: 'str', value: v }); continue; }
    if (/[0-9.]/.test(c)) { let v = ''; while (i < s.length && /[0-9.eE+\-]/.test(s[i]!)) { v += s[i]!; i++; } r.push({ type: 'num', value: v }); continue; }
    if (/[a-zA-Z_]/.test(c)) { let v = ''; while (i < s.length && /[a-zA-Z0-9_.]/.test(s[i]!)) { v += s[i]!; i++; } r.push({ type: 'id', value: v }); continue; }
    r.push({ type: 'unknown', value: c }); i++;
  }
  r.push({ type: 'EOF', value: '' }); return r;
}
class Parser {
  p = 0; constructor(private t: T[]) {}
  peek() { return this.t[this.p] ?? { type: 'EOF', value: '' }; }
  eat(t: string) { const x = this.peek(); if (x.type !== t) throw new Error(`Expected ${t}, got ${x.type}`); this.p++; return x; }
  parse(): any { return this.logicalOr(); }
  logicalOr(): any { let l = this.logicalAnd(); while (this.peek().type === '||') { this.eat('||'); l = { type: 'logical', op: '||', left: l, right: this.logicalAnd() }; } return l; }
  logicalAnd(): any { let l = this.equality(); while (this.peek().type === '&&') { this.eat('&&'); l = { type: 'logical', op: '&&', left: l, right: this.equality() }; } return l; }
  equality(): any { let l = this.comparison(); while (this.peek().type === '==' || this.peek().type === '!=') { const op = this.eat(this.peek().type).type; l = { type: 'compare', op, left: l, right: this.comparison() }; } return l; }
  comparison(): any { let l = this.arith(); while (['<', '>', '<=', '>='].includes(this.peek().type)) { const op = this.eat(this.peek().type).type; l = { type: 'compare', op, left: l, right: this.arith() }; } return l; }
  arith(): any { let l = this.term(); while (this.peek().type === '+' || this.peek().type === '-') { const op = this.eat(this.peek().type).type; l = { type: 'arith', op, left: l, right: this.term() }; } return l; }
  term(): any { let l = this.unary(); while (this.peek().type === '*' || this.peek().type === '/' || this.peek().type === '%') { const op = this.eat(this.peek().type).type; l = { type: 'arith', op, left: l, right: this.unary() }; } return l; }
  unary(): any { if (this.peek().type === '!') { this.eat('!'); return { type: 'unary', op: '!', operand: this.unary() }; } if (this.peek().type === '-') { this.eat('-'); return { type: 'unary', op: '-', operand: this.unary() }; } return this.primary(); }
  primary(): any {
    const t = this.peek();
    if (t.type === 'num') { this.eat('num'); return { type: 'l', v: parseFloat(t.value) }; }
    if (t.type === 'str') { this.eat('str'); return { type: 'l', v: t.value }; }
    if (t.type === 'id') { this.eat('id'); return { type: 'v', n: t.value }; }
    if (t.type === '(') { this.eat('('); const n = this.logicalOr(); this.eat(')'); return n; }
    throw new Error(`Unexpected: ${t.type}`);
  }
}
function evalNode(n: any, ctx: Record<string, unknown>): unknown {
  if (!n || typeof n !== 'object') return n;
  switch (n.type) {
    case 'l': return n.v;
    case 'v': { let v: unknown = ctx; for (const p of n.n.split('.')) { if (v && typeof v === 'object' && p in v) v = (v as any)[p]; else return undefined; } return v; }
    case 'unary': return n.op === '!' ? !evalNode(n.operand, ctx) : n.op === '-' ? -toN(evalNode(n.operand, ctx)) : null;
    case 'arith': { const l = toN(evalNode(n.left, ctx)), r = toN(evalNode(n.right, ctx)); switch (n.op) { case '+': return l + r; case '-': return l - r; case '*': return l * r; case '/': return r ? l / r : 0; case '%': return r ? l % r : 0; default: return null; } }
    case 'compare': { const l = evalNode(n.left, ctx), r = evalNode(n.right, ctx); switch (n.op) { case '==': return deepEq(l, r); case '!=': return !deepEq(l, r); case '<': return toN(l) < toN(r); case '>': return toN(l) > toN(r); case '<=': return toN(l) <= toN(r); case '>=': return toN(l) >= toN(r); default: return null; } }
    case 'logical': { const l = evalNode(n.left, ctx); switch (n.op) { case '||': return l || evalNode(n.right, ctx); case '&&': return l && evalNode(n.right, ctx); default: return null; } }
    default: return null;
  }
}
function toN(v: unknown): number { if (typeof v === 'number') return v; if (typeof v === 'string') return parseFloat(v) || 0; return 0; }
function deepEq(a: unknown, b: unknown): boolean {
  if (a === b) return true;
  if (Array.isArray(a) && Array.isArray(b)) return a.length === b.length && a.every((v, i) => deepEq(v, b[i]));
  return false;
}
