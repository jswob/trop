#!/usr/bin/env node
import { readFileSync } from 'fs';
import { join } from 'path';

interface HookInput {
	session_id: string;
	transcript_path: string;
	cwd: string;
	permission_mode: string;
	prompt: string;
}

interface PromptTriggers {
	keywords?: string[];
	intentPatterns?: string[];
}

interface SkillRule {
	type: 'guardrail' | 'domain';
	enforcement: 'block' | 'suggest' | 'warn';
	priority: 'critical' | 'high' | 'medium' | 'low';
	promptTriggers?: PromptTriggers;
}

interface SkillRules {
	version: string;
	skills: Record<string, SkillRule>;
}

interface MatchedSkill {
	name: string;
	matchType: 'keyword' | 'intent';
	config: SkillRule;
}

async function main() {
	try {
		// Read input from stdin
		const input = readFileSync(0, 'utf-8');
		const data: HookInput = JSON.parse(input);
		const prompt = data.prompt.toLowerCase();

		// Load skill rules
		const projectDir = process.env.CLAUDE_PROJECT_DIR || '$HOME/project';
		const rulesPath = join(projectDir, '.claude', 'skills', 'skill-rules.json');
		const rules: SkillRules = JSON.parse(readFileSync(rulesPath, 'utf-8'));

		const matchedSkills: MatchedSkill[] = [];

		// Check each skill for matches
		for (const [skillName, config] of Object.entries(rules.skills)) {
			const triggers = config.promptTriggers;
			if (!triggers) {
				continue;
			}

			// Keyword matching
			if (triggers.keywords) {
				const keywordMatch = triggers.keywords.some(kw => prompt.includes(kw.toLowerCase()));
				if (keywordMatch) {
					matchedSkills.push({ name: skillName, matchType: 'keyword', config });
					continue;
				}
			}

			// Intent pattern matching
			if (triggers.intentPatterns) {
				const intentMatch = triggers.intentPatterns.some(pattern => {
					const regex = new RegExp(pattern, 'i');
					return regex.test(prompt);
				});
				if (intentMatch) {
					matchedSkills.push({ name: skillName, matchType: 'intent', config });
				}
			}
		}

		// Generate output if matches found
		if (matchedSkills.length > 0) {
			// Group by priority
			const critical = matchedSkills.filter(s => s.config.priority === 'critical');
			const high = matchedSkills.filter(s => s.config.priority === 'high');
			const medium = matchedSkills.filter(s => s.config.priority === 'medium');
			const low = matchedSkills.filter(s => s.config.priority === 'low');

			// Determine if we have high-priority skills
			const hasHighPriority = critical.length > 0 || high.length > 0;

			let output = '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n';
			output += hasHighPriority ? '🎯 SKILL ACTIVATION REQUIRED\n' : '🎯 SKILL ACTIVATION CHECK\n';
			output += '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\n';

			if (critical.length > 0) {
				output += '⛔ MANDATORY SKILLS (MUST use):\n';
				critical.forEach(s => (output += `  → ${s.name}\n`));
				output += '\n';
			}

			if (high.length > 0) {
				output += '📚 REQUIRED SKILLS (high priority):\n';
				high.forEach(s => (output += `  → ${s.name}\n`));
				output += '\n';
			}

			if (medium.length > 0) {
				output += '💡 SUGGESTED SKILLS:\n';
				medium.forEach(s => (output += `  → ${s.name}\n`));
				output += '\n';
			}

			if (low.length > 0) {
				output += '📌 OPTIONAL SKILLS:\n';
				low.forEach(s => (output += `  → ${s.name}\n`));
				output += '\n';
			}

			// Add appropriate footer based on priority
			if (hasHighPriority) {
				output += '⚠️ MANDATORY: You MUST invoke the Skill tool for each\n';
				output += 'skill listed above BEFORE taking any other action.\n';
				output += 'Do NOT skip this step. Do NOT proceed without invoking.\n';
			} else {
				output += 'ACTION: Consider using Skill tool before responding\n';
			}
			output += '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n';

			console.log(output);
		}

		process.exit(0);
	} catch (err) {
		console.error('Error in skill-activation-prompt hook:', err);
		process.exit(1);
	}
}

main().catch(err => {
	console.error('Uncaught error:', err);
	process.exit(1);
});
