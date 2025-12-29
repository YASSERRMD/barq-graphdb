"""
Data models for Barq-GraphDB SDK.
"""

from dataclasses import dataclass, field
from typing import List, Optional


@dataclass
class Node:
    """Represents a node in the graph."""
    
    id: int
    label: str
    embedding: List[float] = field(default_factory=list)
    agent_id: Optional[int] = None
    rule_tags: List[str] = field(default_factory=list)
    timestamp: Optional[int] = None
    has_embedding: bool = False

    def to_dict(self) -> dict:
        """Convert to dictionary for API requests."""
        data = {
            "id": self.id,
            "label": self.label,
        }
        if self.embedding:
            data["embedding"] = self.embedding
        if self.agent_id is not None:
            data["agent_id"] = self.agent_id
        if self.rule_tags:
            data["rule_tags"] = self.rule_tags
        return data

    @classmethod
    def from_dict(cls, data: dict) -> "Node":
        """Create from API response dictionary."""
        return cls(
            id=data["id"],
            label=data.get("label", ""),
            embedding=data.get("embedding", []),
            agent_id=data.get("agent_id"),
            rule_tags=data.get("rule_tags", []),
            timestamp=data.get("timestamp"),
            has_embedding=data.get("has_embedding", False),
        )


@dataclass
class Edge:
    """Represents an edge between nodes."""
    
    from_node: int
    to_node: int
    edge_type: str

    def to_dict(self) -> dict:
        """Convert to dictionary for API requests."""
        return {
            "from": self.from_node,
            "to": self.to_node,
            "edge_type": self.edge_type,
        }


@dataclass
class HybridParams:
    """Parameters for hybrid queries."""
    
    alpha: float = 0.5
    beta: float = 0.5

    def to_dict(self) -> dict:
        """Convert to dictionary for API requests."""
        return {
            "alpha": self.alpha,
            "beta": self.beta,
        }


@dataclass
class HybridResult:
    """Result from a hybrid query."""
    
    id: int
    score: float
    vector_distance: float
    graph_distance: int
    path: List[int]

    @classmethod
    def from_dict(cls, data: dict) -> "HybridResult":
        """Create from API response dictionary."""
        return cls(
            id=data["id"],
            score=data["score"],
            vector_distance=data["vector_distance"],
            graph_distance=data["graph_distance"],
            path=data["path"],
        )


@dataclass
class Decision:
    """Represents an agent decision record."""
    
    id: Optional[int] = None
    agent_id: int = 0
    root_node: int = 0
    path: List[int] = field(default_factory=list)
    score: float = 0.0
    notes: Optional[str] = None
    created_at: Optional[int] = None

    def to_dict(self) -> dict:
        """Convert to dictionary for API requests."""
        data = {
            "agent_id": self.agent_id,
            "root_node": self.root_node,
            "path": self.path,
            "score": self.score,
        }
        if self.notes:
            data["notes"] = self.notes
        return data

    @classmethod
    def from_dict(cls, data: dict) -> "Decision":
        """Create from API response dictionary."""
        return cls(
            id=data.get("id"),
            agent_id=data["agent_id"],
            root_node=data["root_node"],
            path=data["path"],
            score=data["score"],
            notes=data.get("notes"),
            created_at=data.get("created_at"),
        )


@dataclass
class Stats:
    """Database statistics."""
    
    node_count: int
    edge_count: int
    vector_count: int
    decision_count: int

    @classmethod
    def from_dict(cls, data: dict) -> "Stats":
        """Create from API response dictionary."""
        return cls(
            node_count=data["node_count"],
            edge_count=data["edge_count"],
            vector_count=data["vector_count"],
            decision_count=data["decision_count"],
        )
